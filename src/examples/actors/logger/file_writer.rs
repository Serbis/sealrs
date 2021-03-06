use crate::actors::prelude::*;
use std::sync::{Mutex, Arc};
use std::fs;
use match_downcast::*;


pub fn props(file: &str) -> Props {
    Props::new(tsafe!(FileWriter::new(file)))
}

pub struct Write {
    pub text: String,
}

pub struct Ok {
    pub chars_count: usize
}

pub struct FileWriter {
    file: String
}

impl FileWriter {
    pub fn new(file: &str) -> FileWriter {
        FileWriter {
            file: String::from(file)
        }
    }
}

impl Actor for FileWriter {
    fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: Write => {
               fs::write(&self.file, m.text.as_bytes()).unwrap();
               let resp = msg!(Ok { chars_count: m.text.len() });
               ctx.sender.tell(resp, Some(&ctx.self_));
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}

