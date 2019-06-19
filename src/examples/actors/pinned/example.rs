use crate::actors::prelude::*;
use crate::actors::default_dispatcher::DefaultDispatcher;
use crate::examples::actors::pinned::long_worker;
use crate::examples::actors::pinned::long_worker::LongWorker;
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

pub fn run() {
    let mut system = LocalActorSystem::new();

    // Replaces default dispatcher with new instance which uses only one thread
    system.add_dispatcher("default", tsafe!(DefaultDispatcher::new(1)));

    // Create 3 workers with the pinned dispatcher

    let pw0 = Props::new(tsafe!(LongWorker::new("pinned-worker0")))
        .with_dispatcher("pinned");

    let pw1 = Props::new(tsafe!(LongWorker::new("pinned-worker1")))
        .with_dispatcher("pinned");

    let pw2 = Props::new(tsafe!(LongWorker::new("pinned-worker2")))
        .with_dispatcher("pinned");


    let mut pinned_worker0 = system.actor_of(pw0, None);

    let mut pinned_worker1 = system.actor_of(pw1, None);

    let mut pinned_worker2 = system.actor_of(pw2, None);


    // Create 3 workers with the default dispatcher

    let dw0 = Props::new(tsafe!(LongWorker::new("default-worker0")));
    let dw1 = Props::new(tsafe!(LongWorker::new("default-worker1")));
    let dw2 = Props::new(tsafe!(LongWorker::new("default-worker2")));

    let mut default_worker0 = system.actor_of(dw0, None);

    let mut default_worker1 = system.actor_of(dw1, None);

    let mut default_worker2 = system.actor_of(dw2, None);

    // What happens next. We send messages DoLongWork to each worker runned on the default
    // dispatcher. After 16 second, we do similar operation with workers runned on the pinned
    // dispatcher. What you will see in stdout. Workers runned on the default dispatcher will be
    // process messages consistently. Fist message, than second message, and then third. All
    // operation will consume sum of time of all three works - 15 second. This happen because in
    // this example, default despatcher have only one thread in the thread pool. Each message
    // blocks work of all system for message processing time. Workers runned on the pinned
    // dispatcher, will completes they works at the same time, and sum time which will be consumed,
    // will be equal time of one message processing. This happen because each actor have it's own
    // dedicated os thread for messages processing.
    //
    // This example is very good demonstrate, why you must don't do heave tasks in actors which
    // work on thread-pool based dispatchers. And how this omission may cause to very big trouble
    // with performance.

    default_worker0.tell( msg!(long_worker::DoLongWork { time: Duration::from_secs(5) }), None );
    default_worker1.tell( msg!(long_worker::DoLongWork { time: Duration::from_secs(5) }), None );
    default_worker2.tell( msg!(long_worker::DoLongWork { time: Duration::from_secs(5) }), None );

    thread::sleep(Duration::from_secs(16));
    println!("--------------------------------------------------------------");

    pinned_worker0.tell( msg!(long_worker::DoLongWork { time: Duration::from_secs(5) }), None );
    pinned_worker1.tell( msg!(long_worker::DoLongWork { time: Duration::from_secs(5) }), None );
    pinned_worker2.tell( msg!(long_worker::DoLongWork { time: Duration::from_secs(5) }), None );

    thread::park();
}