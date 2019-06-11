use crate::actors::actor::Actor;
use crate::actors::actor_context::ActorContext;
use crate::actors::props::Props;
use std::any::Any;
use std::sync::{Mutex, Arc};
use match_downcast::*;


pub fn props() -> Props {
    Props::new(tsafe!(StdoutWriter::new()))
}

pub struct Write {
    pub text: String,
}

pub struct Ok {
    pub chars_count: usize
}

pub struct StdoutWriter {}

impl StdoutWriter {
    pub fn new() -> StdoutWriter {
        StdoutWriter {}
    }
}

impl Actor for StdoutWriter {
    fn receive(self: &mut Self, msg: &Box<Any + Send>, mut ctx: ActorContext) -> bool {
        match_downcast_ref!(msg, {
            m: Write => {
               println!("{}", m.text);
               let resp = Box::new(Ok { chars_count: m.text.len() });
               ctx.sender.tell(resp, Some(&ctx.self_));
            },
            _ => return false
        });

        true
    }
}