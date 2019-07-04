//! Core of the actor
//!
//! This object is essentially the actor itself. It encapsulate in self actor object, mailbox and
//! dispatcher. Contains references to the actor system and other elements of the system.
//!

use crate::common::tsafe::TSafe;
use crate::actors::dispatcher::Dispatcher;
use crate::actors::mailbox::Mailbox;
use crate::actors::actor_context::ActorContext;
use crate::actors::actor::Actor;
use crate::actors::actor_path::ActorPath;
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::envelope::Envelope;
use crate::actors::abstract_actor_ref::AbstractActorRef;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::local_actor_ref::LocalActorRef;
use crate::actors::watcher::WatchingEvents;
use crate::actors::message::Message;
use crate::actors::error::Error;
use crate::actors::supervision::SupervisionStrategy;
use std::collections::HashMap;
use std::any::Any;

pub struct ActorCell {

    /// Reference to the message dispatcher
    pub dispatcher: TSafe<Dispatcher + Send>,

    /// Actor mailbox
    pub mailbox: TSafe<Mailbox + Send>,

    /// Executor asynchronous block id. Actually this value represents the thread id, on
    /// the actor messages will processed. See default_dispatcher for more info about actor's
    /// async mechanics.
    pub bid: usize,

    /// Object which extends the actor trait and contain application logic
    pub actor: TSafe<Actor + Send>,

    ///  Actor path object that represents the actor position in the actors hierarchy
    pub path: TSafe<ActorPath>,

    /// Reference to the actor system
    pub system: TSafe<AbstractActorSystem + Send>,

    /// Suspend flag. See the suspend method description for more details
    pub suspended: bool,

    /// Stop flag. See the start method description for more details
    pub stopped: bool,

    /// Parent actor cell of actor
    pub parent: Option<TSafe<ActorCell>>,

    /// Actor's childs
    pub childs: HashMap<String, TSafe<ActorCell>>,

    /// Actor's supervision strategy
    pub supervision_strategy: SupervisionStrategy
}

impl ActorCell {

    /// Create new actor cell. This is the internal constructor and should never be used in a
    /// user code.
    pub fn new(system: TSafe<AbstractActorSystem + Send>,
        path: TSafe<ActorPath>,
        actor: TSafe<Actor + Send>,
        bid: usize,
        dispatcher: TSafe<Dispatcher + Send>,
        mailbox: TSafe<Mailbox + Send>,
        parent: Option<TSafe<ActorCell>>,
        supervision_strategy: SupervisionStrategy) -> ActorCell {

        ActorCell {
            actor,
            bid,
            dispatcher,
            mailbox,
            path,
            system,
            suspended: false,
            stopped: true,
            parent,
            childs: HashMap::new(),
            supervision_strategy
        }
    }

    /// Fail handler of actor. This method calls when message handler was completed with error.
    /// Here code realize a supervision strategy of actor and decide what to do next.
    pub fn fail(&mut self, err: Error, boxed_self: TSafe<ActorCell>) -> impl FnOnce() -> () {
        //self.suspended = true;

        let system = self.system.clone();
        let path = self.path.clone();
        let supervision_strategy = self.supervision_strategy.clone();

        move || {
            let self_: ActorRef =  Box::new(LocalActorRef::new(boxed_self.clone(), path));
            let sender = system.lock().unwrap().dead_letters();
            let system = system.clone();

            let ctx = ActorContext::new(sender, self_.clone(), system, boxed_self.clone());

            match supervision_strategy {
                SupervisionStrategy::Resume => {
                    let actor = {
                        let boxed_self = boxed_self.lock().unwrap();
                        boxed_self.actor.clone()
                    };

                    actor.lock().unwrap().pre_fail(ctx, err, SupervisionStrategy::Resume);
                },
                SupervisionStrategy::Restart => {
                    let actor = {
                        let boxed_self = boxed_self.lock().unwrap();
                        boxed_self.actor.clone()
                    };

                    actor.lock().unwrap().pre_fail(ctx, err, SupervisionStrategy::Restart);
                    let f = boxed_self.lock().unwrap().restart(boxed_self.clone());
                    f();
                },
                SupervisionStrategy::Stop => {
                    let actor = {
                        let boxed_self = boxed_self.lock().unwrap();
                        boxed_self.actor.clone()
                    };

                    actor.lock().unwrap().pre_fail(ctx, err, SupervisionStrategy::Stop);
                    let f = boxed_self.lock().unwrap().stop(boxed_self.clone());
                    f();
                },
                SupervisionStrategy::Escalate => {
                    let (actor, parent) = {
                        let boxed_self = boxed_self.lock().unwrap();
                        (boxed_self.actor.clone(), boxed_self.parent.clone())
                    };

                    actor.lock().unwrap().pre_fail(ctx, err.clone(), SupervisionStrategy::Escalate);
                    let parent = parent.unwrap();
                    let f = parent.lock().unwrap().fail(err, parent.clone());
                    f();
                }
            };
        }
    }

    /// Starts the actor. Creates him context, obtain bid form the dispatcher, run preStart hook
    /// and permits message receiving through dropping the stopped flag.
    pub fn start(self: &mut Self, boxed_self: TSafe<ActorCell>) -> impl FnOnce() -> () {
        self.bid = self.dispatcher.lock().unwrap().obtain_bid();
        //println!("Bid = {}", self.bid);

        let self_ =  Box::new(LocalActorRef::new(boxed_self.clone(), self.path.clone()));
        let sender = self.system.lock().unwrap().dead_letters();
        let system = self.system.clone();

        let ctx = ActorContext::new(sender, self_, system, boxed_self.clone());

        move || {
            let actor = {
                let boxed_self = boxed_self.lock().unwrap();
                boxed_self.actor.clone()
            };

            actor.lock().unwrap().pre_start(ctx);
            boxed_self.lock().unwrap().stopped = false;
        }
    }

    pub fn restart(self: &mut Self, boxed_self: TSafe<ActorCell>) -> impl FnOnce() -> ()  {
        let self_ =  Box::new(LocalActorRef::new(boxed_self.clone(), self.path.clone()));
        let sender = self.system.lock().unwrap().dead_letters();
        let system = self.system.clone();

        let ctx = ActorContext::new(sender, self_, system, boxed_self.clone());

        move || {
            let f = {
                let mut boxed_self_o = boxed_self.lock().unwrap();
                let f = boxed_self_o.stop(boxed_self.clone());
                f
            };
            f();

            let f = {
                let mut boxed_self_o = boxed_self.lock().unwrap();
                let f = boxed_self_o.start(boxed_self.clone());
                f
            };
            f();

            let (actor, system) = {
                let boxed_self = boxed_self.lock().unwrap();
                (boxed_self.actor.clone(), boxed_self.system.clone())
            };

            actor.lock().unwrap().post_restart(ctx);
        }
    }

    /// Stops the actor. Prohibits receiving new messages and calls the postStop hook.
    pub fn stop(self: &mut Self, boxed_self: TSafe<ActorCell>) -> impl FnOnce() -> () {
        self.stopped = true;

        //FIXME this is potential memory leak place! What happen if an actor is stopped but his mailbox is not empty?
        //self.mailbox.lock().unwrap().clean_up();

        // Stop all childs
        for (path, cell) in self.childs.iter() {
            let cell = cell.clone();
            let boxed_cell = cell.clone();
            let  f = cell.lock().unwrap().stop(boxed_cell);
            f();
        }
        self.childs.clear();

        let self_: ActorRef =  Box::new(LocalActorRef::new(boxed_self.clone(), self.path.clone()));
        let sender = self.system.lock().unwrap().dead_letters();
        let system = self.system.clone();

        let ctx = ActorContext::new(sender, self_.clone(), system, boxed_self.clone());

        move || {
            let (actor, system) = {
                let boxed_self = boxed_self.lock().unwrap();
                (boxed_self.actor.clone(), boxed_self.system.clone())
            };

            actor.lock().unwrap().post_stop(ctx);
            system.lock().unwrap().register_watch_event(&self_, WatchingEvents::Terminated);

        }

    }

    /// Suspends the actor. Prohibits receiving new messages.
    pub fn suspend(self: &mut Self) {
        self.suspended = true;
    }

    /// Sends the message to the actor. Creates new envelope with the message and indicates to
    /// dispatcher to schedule execution of this envelope. Message sends to the actors may be done,
    /// only if flags suspended and stopped will be dropped. Otherwise, the message will be dropped
    /// to deadLetter.
    pub fn send(self: &mut Self,
                boxed_self: &TSafe<ActorCell>,
                msg: Message,
                rself: Option<ActorRef>,
                to_ref: Box<AbstractActorRef + Send>) {

        // If cell does not receive new messages, drops message to the deadLetter
        if self.stopped || self.suspended {
            let mut dead_letters = self.system.lock().unwrap().dead_letters();
            dead_letters.cell().lock().unwrap().send(&dead_letters.cell(),
                                                   msg, rself,
                                                   to_ref);
        } else {
            let envelope = Envelope::new(
                msg,
                rself,
                to_ref,
                self.system.clone());


            self.dispatcher.lock().unwrap().dispatch(
                boxed_self.clone(),
                self.bid,
                self.mailbox.clone(),
                self.actor.clone(), envelope);
        }
    }

    /// Performs action identical to the send method do, but with ignoring state of the stopping
    /// flags.
    pub fn force_send(self: &mut Self,
                      boxed_self: TSafe<ActorCell>,
                      msg: Message,
                      rself: Option<Box<AbstractActorRef + Send>>,
                      to_ref: Box<AbstractActorRef + Send>) {

        let envelope = Envelope::new(
            msg,
            rself,
            to_ref,
            self.system.clone());

        self.dispatcher.lock().unwrap().dispatch(
            boxed_self,
            self.bid,
            self.mailbox.clone(),
            self.actor.clone(), envelope);
    }
}

//impl Drop for ActorCell {
//    fn drop(&mut self) {
//        println!("ActorCell dropped")
//    }
//}
//


// Attentions!!! This object does't do be cloned. Cloned must by on the boxed (TSave) value of the
// cell.