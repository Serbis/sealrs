use crate::actors::prelude::*;
use crate::examples::actors::logger::logger;
use crate::examples::actors::logger::stdout_writer;
use crate::examples::actors::logger::file_writer;
use std::sync::{Mutex, Arc};
use std::thread;

pub fn run() {
    let mut system = LocalActorSystem::new();

    let mut logger =  {
        let mut system = system.lock().unwrap();
        let file_writer = system.actor_of(file_writer::props("/tmp/log"), Some("file_writer"));
        let stdout_writer = system.actor_of(stdout_writer::props(), Some("stdout_writer"));
        system.actor_of(logger::props(file_writer, stdout_writer), Some("logger"))
    };

    logger.tell(msg!(logger::Log { text: String::from("To file log"), target: logger::LogTarget::File }), None);
    logger.tell(msg!(logger::Log { text: String::from("To stdout log"), target: logger::LogTarget::StdOut }), None);

    thread::park();
}