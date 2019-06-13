//! Actor trait
//!
//! This trait must be realized by the structure, which want to be involved to the system as
//! a particular actor.
use crate::actors::actor_context::ActorContext;
use crate::actors::message::Message;
use std::any::Any;

pub trait Actor {
    fn pre_start(&mut self, _ctx: ActorContext) {}
    fn post_stop(&mut self, _ctx: ActorContext) {}
    fn receive(&mut self, msg: Message, ctx: ActorContext) -> bool;
    fn as_any(&mut self) -> &Any {
        panic!()
    }
}

/// Service message. Stops the actor which will receive him. See  actors lifetime management
/// articles in the main doc, for more details, about how this message works.
pub struct PoisonPill {}
