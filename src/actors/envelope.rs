//! Internal representation of a message
//!
//! Contains the message itself and elements of context.
use crate::actors::abstract_actor_ref::AbstractActorRef;
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::message::Message;
use crate::common::tsafe::TSafe;
use std::any::Any;


pub struct Envelope {

    /// Boxed message
    pub message: Message,

    /// Who send this message
    pub sender: Option<Box<AbstractActorRef + Send>>,

    /// Who must receive this message
    pub receiver: Box<AbstractActorRef + Send>,

    /// Link to the actor system
    pub system: TSafe<AbstractActorSystem + Send>
}

impl Envelope {
    pub fn new(message: Message, sender: Option<Box<AbstractActorRef + Send>>, receiver: Box<AbstractActorRef + Send>, system: TSafe<AbstractActorSystem + Send>) -> Envelope {
        Envelope {
            message,
            sender,
            receiver,
            system
        }
    }
}