use crate::actors::local_actor_system::LocalActorSystem;
use crate::actors::actor_ref_factory::ActorRefFactory;
use super::watcher;
use super::target;
use std::time::Duration;
use std::thread;

pub fn run() {
    let mut system = LocalActorSystem::new();

    // Creates actor which will be watcher be the watcher actor
    let mut target = system.lock().unwrap()
        .actor_of(target::props(), Some("target"));

    // Create actor which will watch the rarget actor
    let mut watcher = system.lock().unwrap()
        .actor_of(watcher::props(target.clone()), None);


    //Sleep for significant time for all actors may be complete it's initialization
    thread::sleep(Duration::from_secs(3));

    // Stops the target actor. After that, watcher will receive Terminated message and print the message
    // about it
    system.lock().unwrap().stop(&mut target);

    thread::sleep(Duration::from_secs(1));

    system.lock().unwrap().stop(&mut watcher);

    thread::park();
}