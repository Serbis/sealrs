use crate::actors::prelude::*;
use crate::examples::actors::supervision::b_actor;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props() -> Props {
    Props::new(tsafe!(AActor::new()))
        .with_supervision_strategy(SupervisionStrategy::Restart)
}

pub struct Init {}

pub struct AActor {
    printed_chars: usize
}

impl AActor {
    pub fn new() -> AActor {
        AActor {
            printed_chars: 0
        }
    }
}

impl Actor for AActor {

    fn pre_start(self: &mut Self, mut ctx: ActorContext) {
        println!("AActor is started at path '{}'", ctx.self_.path());
        ctx.actor_of(b_actor::props(), Some("b"));
    }

    fn post_stop(self: &mut Self, ctx: ActorContext) {
        println!("AActor is stopped")
    }

    fn pre_fail(&mut self, _ctx: ActorContext, err: Error, strategy: SupervisionStrategy) {
        println!("AActor failed in '{:?}' strategy", strategy);
    }

    fn post_restart(&mut self, _ctx: ActorContext) {
        println!("AActor was restarted");
    }

    fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: Init => {
                let mut b = ctx.actor_of(b_actor::props(), Some("b"));
                b.tell(msg!(b_actor::Init {}), None);
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}