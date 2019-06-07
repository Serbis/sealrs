//! Actor trait
//!
//! This trait must be realized by the structure, which want to be involved to the system as
//! a particular actor.
use crate::actors::actor_context::ActorContext;
use std::any::Any;

pub trait Actor {
    fn pre_start(self: &mut Self, _ctx: ActorContext) {}
    fn post_stop(self: &mut Self, _ctx: ActorContext) {}
    fn receive(self: &mut Self, msg: &Box<Any + Send>, ctx: ActorContext) -> bool;
    fn as_any(self: &Self) -> &Any {
        panic!()
    }
}

/// Service message. Stops the actor which will receive him. See  actors lifetime management
/// articles in the main doc, for more details, about how this message works.
pub struct PoisonPill {}
