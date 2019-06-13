use crate::actors::prelude::*;
use crate::examples::actors::basic::basic_actor;
use std::sync::{Mutex, Arc};
use std::thread;

pub fn run() {
    let mut system = LocalActorSystem::new();

    let mut printer = system.lock().unwrap()
        .actor_of(basic_actor::props(), Some("printer"));

    let msg = msg!(basic_actor::Print { text: String::from("Hello world!") });
    printer.tell(msg, None);

    thread::sleep_ms(1000);

    system.lock().unwrap().terminate();
}