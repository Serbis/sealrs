//! Message processing context
//!
//! This object will constructs for each new message received by an actor
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::actor_cell::ActorCell;
use crate::actors::props::Props;
use crate::common::tsafe::TSafe;
use std::sync::MutexGuard;

//pub struct WrappedCell {
//    refed: Option<&ActorCell>,
//    safe: Option<>
//}

pub struct ActorContext {

    /// Who send the current message
    pub sender: ActorRef,

    /// Own actor reference
    pub self_: ActorRef,

    /// Actor system where actor is work
    pub system: TSafe<AbstractActorSystem + Send>


}

impl ActorContext {
    pub fn new(sender: ActorRef,
               self_: ActorRef,
               system: TSafe<AbstractActorSystem + Send>,) -> ActorContext
    {
        ActorContext {
            sender,
            self_,
            system,
        }
    }

    pub fn system(&self) -> MutexGuard<AbstractActorSystem + Send + 'static> {
        self.system.lock().unwrap()
    }
}

impl ActorRefFactory for ActorContext {
    fn actor_of(self: &mut Self, props: Props, name: Option<&str>) -> ActorRef {
        unimplemented!()
    }

    fn stop(self: &mut Self, aref: &mut ActorRef) {
        unimplemented!()
    }

    fn dead_letters(self: &mut Self) -> ActorRef {
        unimplemented!()
    }

    fn watch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        unimplemented!()
    }

    fn unwatch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        unimplemented!()
    }
}