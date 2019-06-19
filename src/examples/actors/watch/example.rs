use crate::actors::prelude::*;
use super::watcher;
use super::target;
use std::time::Duration;
use std::thread;

pub fn run() {
    let mut system = LocalActorSystem::new();

    // Creates actor which will be watcher be the watcher actor
    let mut target = system
        .actor_of(target::props(), Some("target"));

    // Create actor which will watch the rarget actor
    let mut watcher = system
        .actor_of(watcher::props(target.clone()), None);


    //Sleep for significant time for all actors may be complete it's initialization
    thread::sleep(Duration::from_secs(3));

    // Stops the target actor. After that, watcher will receive Terminated message and print the message
    // about it
    system.stop(&mut target);

    thread::sleep(Duration::from_secs(1));

    system.stop(&mut watcher);

    thread::park();
}