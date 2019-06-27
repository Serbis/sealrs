use crate::actors::prelude::*;
use std::sync::{Mutex, Arc};

pub fn props() -> Props {
    Props::new(tsafe!(Target::new()))
}

//This actor does nothing. It's simply stub for demonstrate mechanic of watching
pub struct Target {}

impl Target {
    pub fn new() -> Target {
        Target {}
    }
}

impl Actor for Target {

    fn receive(self: &mut Self, _msg: Message, _ctx: ActorContext) -> HandleResult {
        Ok(false)
    }
}