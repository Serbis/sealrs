use crate::actors::prelude::*;
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
    fn receive(&mut self, msg: Message, mut ctx: ActorContext) -> bool {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: Write => {
               println!("{}", m.text);
               let resp = msg!(Ok { chars_count: m.text.len() });
               ctx.sender.tell(resp, Some(&ctx.self_));
            },
            _ => return false
        });

        true
    }
}