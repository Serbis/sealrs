//! Empty actor stub for service needs
//!
//! For example, he is used for construct ActorCell of the DeadLetters.
use crate::actors::actor::Actor;
use crate::actors::message::Message;
use crate::actors::actor_context::ActorContext;
use std::any::Any;

pub struct SyntheticActor {}

impl Actor for SyntheticActor {
    fn receive(self: &mut Self, _msg: Message, _ctx: ActorContext) -> bool {
        unimplemented!()
    }
}