//! Empty actor stub for service needs
//!
//! For example, he is used for construct ActorCell of the DeadLetters.
use crate::actors::actor::Actor;
use std::any::Any;
use crate::actors::actor_context::ActorContext;

pub struct SyntheticActor {}

impl Actor for SyntheticActor {
    fn receive(self: &mut Self, _msg: &Box<Any + Send>, _ctx: ActorContext) -> bool {
        unimplemented!()
    }
}