use crate::actors::prelude::*;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props() -> Props {
    Props::new(tsafe!(BasicActor::new()))
}

pub struct Print {
    pub text: String,
}

pub struct BasicActor {
    printed_chars: usize
}

impl BasicActor {
    pub fn new() -> BasicActor {
        BasicActor {
            printed_chars: 0
        }
    }
}

impl Actor for BasicActor {

    fn pre_start(self: &mut Self, _ctx: ActorContext) {
        println!("BasicActor is started")
    }

    fn post_stop(self: &mut Self, _ctx: ActorContext) {
        println!("BasicActor is stopped")
    }

    fn receive(self: &mut Self, msg: Message, _ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: Print => {
                self.printed_chars = self.printed_chars + m.text.len();
                println!("{}", m.text);
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}