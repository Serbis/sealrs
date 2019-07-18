use crate::actors::prelude::*;
use crate::examples::actors::supervision::a_actor;
use crate::examples::actors::supervision::c_actor;
use crate::futures::future::WrappedFuture;
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

pub fn run() {

    // In this example presents various supervision operations. Here created the chain of three
    // actors AActor->BActor->CActor. BActor and CActor have Escalate supervision strategy.
    // AActor have Restart strategy. After sending FailMe message to the CActor, he will cause
    // error, which lead to the supervision actions from the system. You may change strategy of
    // each separate actor, for see, how this hierarchy will behave. Also here is demonstrated
    // how to use the actor_select method, and what occurs if you try to stop an actor, which
    // have a childs.


    let mut system = LocalActorSystem::new();

    let mut a = system
        .actor_of(a_actor::props(), Some("a"));

    // Wail while all actor's hierarchy will be initialized
    thread::sleep(Duration::from_secs(1));

    // Select actor c
    let mut selection = system.actor_select("/root/a/b/c");
    let c = &mut selection[0];

    println!("Selected actor with path '{}'", c.path());

    // Send message to the c actor
    c.tell(msg!(c_actor::HelloToC {}), None);

    // Wail while HelloToC will be delivered
    thread::sleep(Duration::from_secs(1));

    // This message  cause to fail CActor
    c.tell(msg!(c_actor::FailMe {}), None);

    // Stops the AActor - this action cause to recursive stopping of all actors tree
    //system.stop(&mut a);

    // Stops the system - this action cause to stop the Root Guardian actor, which will lead to
    // stop of all existed actors
    //system.terminate();

    thread::park()
}