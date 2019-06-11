use crate::actors::local_actor_system::LocalActorSystem;
use crate::actors::actor_ref_factory::ActorRefFactory;
use super::ticker;
use std::thread;

pub fn run() {
    let mut system = LocalActorSystem::new();

    let mut _ticker = system.lock().unwrap()
        .actor_of(ticker::props(), None);

    thread::park();
}