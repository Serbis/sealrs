use crate::actors::prelude::*;
use crate::examples::actors::supervision::c_actor;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props() -> Props {
    Props::new(tsafe!(BActor::new()))
        .with_supervision_strategy(SupervisionStrategy::Escalate)
}

pub struct Init {}

pub struct BActor {
    printed_chars: usize
}

impl BActor {
    pub fn new() -> BActor {
        BActor {
            printed_chars: 0
        }
    }
}

impl Actor for BActor {

    fn pre_start(self: &mut Self, mut ctx: ActorContext) {
        println!("BActor is started at path '{}'", ctx.self_.path());
        ctx.actor_of(c_actor::props(), Some("c"));
    }

    fn post_stop(self: &mut Self, _ctx: ActorContext) {
        println!("BActor is stopped")
    }

    fn pre_fail(&mut self, _ctx: ActorContext, err: Error, strategy: SupervisionStrategy) {
        println!("BActor failed in '{:?}' strategy", strategy);
    }

    fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: Init => {
                let c = ctx.actor_of(c_actor::props(), Some("c"));
                //b.tell(msg!(b_actor::Init {}), None);
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}