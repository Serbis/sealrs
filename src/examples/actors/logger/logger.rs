use crate::actors::prelude::*;
use crate::examples::actors::logger::file_writer;
use crate::examples::actors::logger::stdout_writer;
use std::any::Any;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props(file_writer: ActorRef, stdout_writer: ActorRef) -> Props {
    Props::new(tsafe!(Logger::new(file_writer, stdout_writer)))
}

pub enum LogTarget {
    StdOut, File
}

pub struct Log {
    pub text: String,
    pub target: LogTarget
}

pub struct Logger {
    chars_counter: usize,
    file_writer: ActorRef,
    stdout_writer: ActorRef,
}

impl Logger {
    pub fn new(file_writer: ActorRef, stdout_writer: ActorRef) -> Logger {
        Logger {
            chars_counter: 0,
            file_writer,
            stdout_writer
        }
    }
}

impl Actor for Logger {
    fn receive(self: &mut Self, msg: Message, ctx: ActorContext) -> bool {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: Log => {
                match m.target {
                    LogTarget::File => {
                        let msg = msg!(file_writer::Write { text: m.text.to_string() });
                        self.file_writer.tell(msg , Some(&ctx.self_))
                    },
                    LogTarget::StdOut => {
                        let msg = msg!(stdout_writer::Write { text: m.text.to_string() });
                        self.stdout_writer.tell(msg, Some(&ctx.self_))
                    }
                };
            },
            m: file_writer::Ok => {
                println!("File logger write '{}' chars", m.chars_count);
                self.chars_counter = self.chars_counter + m.chars_count;
            },
            m: stdout_writer::Ok => {
                 println!("Stout logger write '{}' chars", m.chars_count);
                self.chars_counter = self.chars_counter + m.chars_count;
            },
            _ => return false
        });

        true
    }
}