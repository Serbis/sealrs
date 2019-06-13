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
}

impl ActorCell {

    /// Create new actor cell. This is the internal constructor and should never be used in a
    /// user code.
    pub fn new(system: TSafe<AbstractActorSystem + Send>,
        path: TSafe<ActorPath>,
        actor: TSafe<Actor + Send>,
        bid: usize,
        dispatcher: TSafe<Dispatcher + Send>,
        mailbox: TSafe<Mailbox + Send>) -> ActorCell {

        ActorCell {
            actor,
            bid,
            dispatcher,
            mailbox,
            path,
            system,
            suspended: false,
            stopped: true
        }
    }

    /// Starts the actor. Creates him context, obtain bid form the dispatcher, run preStart hook
    /// and permits message receiving through dropping the stopped flag.
    pub fn start(self: &mut Self, boxed_self: TSafe<ActorCell>) {
        self.bid = self.dispatcher.lock().unwrap().obtain_bid();
        //println!("Bid = {}", self.bid);

        let self_ =  Box::new(LocalActorRef::new(boxed_self, self.path.clone()));
        let sender = self.system.lock().unwrap().dead_letters();
        let system = self.system.clone();

        let ctx = ActorContext::new(sender, self_, system);
        self.actor.lock().unwrap().pre_start(ctx);
        self.stopped = false;
    }

    /// Stops the actor. Prohibits receiving new messages and calls the postStop hook.
    pub fn stop(self: &mut Self, boxed_self: TSafe<ActorCell>) {
        self.stopped = true;

        let self_: ActorRef =  Box::new(LocalActorRef::new(boxed_self, self.path.clone()));
        let sender = self.system.lock().unwrap().dead_letters();
        let system = self.system.clone();

        let ctx = ActorContext::new(sender, self_.clone(), system);
        self.actor.lock().unwrap().post_stop(ctx);
        self.system.lock().unwrap().register_watch_event(&self_, WatchingEvents::Terminated);
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



// Attentions!!! This object does't do be cloned. Cloned must by on the boxed (TSave) value of the
// cell.