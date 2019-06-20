use crate::actors::prelude::*;
use crate::examples::actors::stash::behactor;
use std::sync::{Mutex, Arc};
use std::thread;

pub fn run() {

    // This example demonstrates multi-behaviors actor with using the Stash. Here to the actor
    // sends four messages. MessageA, MessageB and MessageC will be processed in state 0.
    // MessageD will be processed in state 1. At the same time MessageA switch behavior of the
    // actor from type 0 to type 1 and MessageD switch from type 1 to type 0. In behavior type 1
    // all messages except MessageD will be stashed. In MessageD handler, all early stashed
    // message will be unstashed. Based on this algorithm, despite fact that you will send
    // message in sequenced order (A, B, C and D), they will be processed in other order A, D,
    // B and C.

    let mut system = LocalActorSystem::new();

    let mut actor = system
        .actor_of(behactor::props(), None);

    actor.tell(msg!(behactor::MessageA {}), None);
    actor.tell(msg!(behactor::MessageB {}), None);
    actor.tell(msg!(behactor::MessageC {}), None);
    actor.tell(msg!(behactor::MessageD {}), None);

    thread::park()
}