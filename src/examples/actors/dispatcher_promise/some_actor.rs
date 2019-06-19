use crate::actors::prelude::*;
use crate::futures::future::{Future, WrappedFuture};
use crate::executors::executor::{Executor, ExecutorTask};
use crate::actors::dispatcher::Dispatcher;
use crate::common::tsafe::TSafe;
use std::any::Any;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props() -> Props {
    Props::new(tsafe!(SomeActor::new()))
}

pub struct SomeMsg {}

pub struct SomeActor {}

impl SomeActor {
    pub fn new() -> SomeActor {
        SomeActor {}
    }
}

impl Actor for SomeActor {

    fn receive(self: &mut Self, msg: Message, ctx: ActorContext) -> bool {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: SomeMsg => {
                let dispatcher = ctx.system().get_executor("default");
                let _fut: WrappedFuture<(), ()> = Future::asyncp(|| {
                    println!("I am the async promise executed on the actor system dispatcher");
                    Ok(())
                }, dispatcher);
            },
            _ => return false
        });

        true
    }
}

