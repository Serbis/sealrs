use crate::actors::prelude::*;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props() -> Props {
    Props::new(tsafe!(CActor::new()))
        .with_supervision_strategy(SupervisionStrategy::Escalate)
}

pub struct HelloToC {}

pub struct FailMe {}

pub struct CActor {
    printed_chars: usize
}

impl CActor {
    pub fn new() -> CActor {
        CActor {
            printed_chars: 0
        }
    }
}

impl Actor for CActor {

    fn pre_start(self: &mut Self, ctx: ActorContext) {
        println!("CActor is started at path '{}'", ctx.self_.path())
    }

    fn post_stop(self: &mut Self, _ctx: ActorContext) {
        println!("CActor is stopped")
    }

    fn pre_fail(&mut self, _ctx: ActorContext, err: Error, strategy: SupervisionStrategy) {
        println!("CActor failed in '{:?}' strategy", strategy);
    }

    fn receive(self: &mut Self, msg: Message, _ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: HelloToC => {
                println!("Hello to some from the outside world!")
            },
            m: FailMe => {
                println!("CActor receive FailMe");
                return Err(err!("some err"))
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}