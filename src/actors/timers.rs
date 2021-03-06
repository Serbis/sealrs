//! Timers object for planning messages sending.
//!
//! This object is intermediate code for interact with scheduler of actor system . He presents
//! abstractions, for send a messages with delay and periodically through interval. For more
//! details, see the module level doc, section Timers.

use crate::common::tsafe::TSafe;
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::message::Message;
use crate::actors::scheduler::TaskGuard;
use crate::actors::abstract_actor_ref::ActorRef;
use std::collections::HashMap;
use std::time::Duration;
use std::sync::mpsc::channel;

pub type Timers = Box<AbstractTimers + Send>;

pub trait AbstractTimers {
    fn start_single(&mut self, key: u32, self_: &ActorRef, to: &ActorRef, delay: Duration, msg: Message);
    fn start_periodic(&mut self, key: u32, self_: &ActorRef, to: &ActorRef, interval: Duration, msg: Box<'static + Fn() -> Message + Send>);
    fn cancel(&mut self, key: u32);
    fn cancel_all(&mut self);
}

pub struct RealTimers {
    system: TSafe<AbstractActorSystem + Send>,
    tasks: HashMap<u32, TaskGuard>
}

impl RealTimers {
    pub fn new(system: TSafe<AbstractActorSystem + Send>) -> Timers {
        let r = RealTimers {
            system,
            tasks: HashMap::new()
        };

        Box::new(r)
    }
}

impl AbstractTimers for RealTimers {

    /// Starts single timer task. Accept as args - key, refs to self and receiver of message, delay
    /// and message for send. For example see module level doc or example actor in the examples
    /// module.
    fn start_single(&mut self, key: u32, self_: &ActorRef, to: &ActorRef, delay: Duration, msg: Message)
    {
        let (msg_sender, msg_receiver) = channel();
        let (self_sender, self_receiver) = channel();
        let (to_sender, to_receiver) = channel();


        let scheduler = self.system.lock().unwrap().get_scheduler();
        let guard = scheduler.lock().unwrap().schedule_once(delay,move || {
            let msg = msg_receiver.recv().unwrap();
            let self_ = self_receiver.recv().unwrap();
            let mut to: ActorRef = to_receiver.recv().unwrap();

            to.tell(msg, Some(&self_));
        });

        self.tasks.insert(key, guard);

        msg_sender.send(msg);
        self_sender.send((*self_).clone());
        to_sender.send((*to).clone());
    }

    /// Starts single timer task. Accept as args - key, refs to self and receiver of message, delay
    /// and closure which produce a message for send. For example see the module level doc or
    /// example actor in the examples module.
    fn start_periodic(&mut self, key: u32, self_: &ActorRef, to: &ActorRef, interval: Duration, msg: Box<'static + Fn() -> Message + Send>)
    {
        let (msg_sender, msg_receiver) = channel();
        let (self_sender, self_receiver) = channel();
        let (to_sender, to_receiver) = channel();

        let msg_sender_clone = msg_sender.clone();
        let self_sender_clone = self_sender.clone();
        let to_sender_clone = to_sender.clone();

        let scheduler = self.system.lock().unwrap().get_scheduler();
        let guard = scheduler.lock().unwrap().schedule_periodic(interval,move || {
            let msg: Box<'static + Fn() -> Message + Send> = msg_receiver.recv().unwrap();
            let self_: ActorRef = self_receiver.recv().unwrap();
            let mut to: ActorRef = to_receiver.recv().unwrap();

            to.tell(msg(), Some(&self_));

            msg_sender_clone.send(msg);
            self_sender_clone.send(self_);
            to_sender_clone.send(to);
        });

        self.tasks.insert(key, guard);

        msg_sender.send(msg);
        self_sender.send((*self_).clone());
        to_sender.send((*to).clone());
    }

    /// Cancel timer by it's key
    fn cancel(&mut self, key: u32) {
        self.tasks.remove(&key);
    }

    /// Cancels all timers
    fn cancel_all(&mut self) {
        self.tasks.clear();
    }
}

pub struct StubTimers {}

impl StubTimers {
    pub fn new() -> Timers {
        Box::new(StubTimers {})
    }
}

impl AbstractTimers for StubTimers {
    fn start_single(&mut self, key: u32, self_: &ActorRef, to: &ActorRef, delay: Duration, msg: Message) {
        unimplemented!()
    }

    fn start_periodic(&mut self, key: u32, self_: &ActorRef, to: &ActorRef, interval: Duration, msg: Box<'static + Fn() -> Message + Send>)
    {
        unimplemented!()
    }

    fn cancel(&mut self, key: u32) {}

    fn cancel_all(&mut self) {}
}