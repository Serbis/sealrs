//! Message processing context
//!
//! This object will constructs for each new message received by an actor
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::abstract_actor_ref::AbstractActorRef;
use crate::common::tsafe::TSafe;

pub struct ActorContext {

    /// Who send the current message
    pub sender: Box<AbstractActorRef + Send>,

    /// Own actor reference
    pub self_: Box<AbstractActorRef + Send>,

    /// Actor system where actor is work
    pub system: TSafe<AbstractActorSystem + Send>
}

impl ActorContext {
    pub fn new(sender: Box<AbstractActorRef + Send>,
               self_: Box<AbstractActorRef + Send>,
               system: TSafe<AbstractActorSystem + Send>) -> ActorContext
    {
        ActorContext {
            sender,
            self_,
            system,
        }
    }
}