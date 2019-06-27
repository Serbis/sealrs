use crate::actors::prelude::*;
use crate::examples::actors::fsm::fsm_actor;
use std::sync::{Mutex, Arc};
use std::thread;

struct OtherMsg {}

pub fn run() {
    // This example demonstrate technical structure of actor based on the fsm module. He is do
    // not anything useful and simple demonstrate how fsm is work.

    let mut system = LocalActorSystem::new();

    let mut actor = system
        .actor_of(fsm_actor::props(), None);

    actor.tell(msg!(fsm_actor::MessageA { v: 100 }), None);
    actor.tell(msg!(fsm_actor::MessageB { }), None);
    actor.tell(msg!(fsm_actor::MessageOther { }), None);

    thread::park();
}