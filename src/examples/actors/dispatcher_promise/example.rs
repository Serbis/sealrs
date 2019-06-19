use crate::actors::prelude::*;
use crate::examples::actors::dispatcher_promise::some_actor;
use std::sync::{Mutex, Arc};
use std::thread;

struct OtherMsg {}

pub fn run() {
    // In this example you can see, how you may use dispatcher of actor system as executor,
    // and plan async promise on him. All logic placed in receive method of SomeActor actor.

    let mut system = LocalActorSystem::new();

    let mut actor = system
        .actor_of(some_actor::props(), None);

    actor.tell(msg!(some_actor::SomeMsg {}), None);

    thread::park();
}