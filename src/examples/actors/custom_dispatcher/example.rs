use crate::actors::prelude::*;
use crate::examples::actors::basic::basic_actor;
use super::my_dispatcher::MyDispatcher;
use std::sync::{Mutex, Arc};
use std::thread;

struct OtherMsg {}

pub fn run() {
    // This example demonstrates how you may implement your own dispatcher. MyDispatcher is the
    // simplified clone of the DefaultDispatcher, which contain logging of every stage of him work.
    // This dispatcher is added to the actor system and on him will runs the basic_actor from
    // the basic example. This example work in a similar way as the basic example, but this adds
    // some additional operations, for demonstrate, how dispatcher is reacts to them.

    // Creating new dispatcher instance
    let my_dispatcher = MyDispatcher::new(1);


    let mut system = LocalActorSystem::new();

    // Registration of the early created dispatcher in the actor system
    system.add_dispatcher("my-dispatcher", tsafe!(my_dispatcher));

    // Creating the actor with the custom dispatcher
    let custom_props = Props::new(tsafe!(basic_actor::BasicActor::new()))
        .with_dispatcher("my-dispatcher");

    let mut printer = system.actor_of(custom_props, None);

    // Sends the message which will be handled
    let msg = msg!(basic_actor::Print { text: String::from("Hello world!") });
    printer.tell(msg, None);

    // Sends the message which will does't be handled
    let msg = msg!(OtherMsg {});
    printer.tell(msg, None);

    thread::sleep_ms(1000);

    // Stops the actor
    system.stop(&mut printer);

    thread::sleep_ms(1000);

    // Terminate the actor system
    system.terminate();

    thread::park();
}