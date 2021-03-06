use crate::actors::prelude::*;
use super::ticker;
use std::thread;

pub fn run() {
    let mut system = LocalActorSystem::new();

    let mut _ticker = system
        .actor_of(ticker::props(), None);

    thread::park();
}