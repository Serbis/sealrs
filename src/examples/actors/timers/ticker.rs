use crate::common::tsafe::TSafe;
use crate::actors::actor::Actor;
use crate::actors::actor_context::ActorContext;
use crate::actors::props::Props;
use crate::actors::timers::Timers;
use std::any::Any;
use std::sync::{Mutex, Arc};
use match_downcast::*;
use std::time::Duration;

pub fn props() -> Props {
    Props::new(tsafe!(Ticker::new()))
}

pub struct SingleTick {}

pub struct PeriodicTick {}

pub struct Ticker {
    timers: Option<TSafe<Timers>>,
    ticks: u32
}

impl Ticker {
    pub fn new() -> Ticker {
        Ticker {
            timers: None,
            ticks: 0,
        }
    }
}

impl Actor for Ticker {

    fn pre_start(self: &mut Self, ctx: ActorContext) {
        let mut timers = Timers::new(ctx.system.clone());

        timers.start_single(
            0,
            ctx.self_.clone(),
            ctx.self_.clone(),
            Duration::from_secs(1),
            Box::new(SingleTick {}));

        timers.start_periodic(
            1,
            ctx.self_.clone(),
            ctx.self_.clone(),
            Duration::from_secs(2),
            || Box::new(PeriodicTick {}));


        self.timers = Some(tsafe!(timers));
    }

    fn post_stop(&mut self, _ctx: ActorContext) {
        self.timers.as_ref().unwrap().lock().unwrap().cancel_all();
    }


    fn receive(self: &mut Self, msg: &Box<Any + Send>, ctx: ActorContext) -> bool {
        match_downcast_ref!(msg, {
            m: SingleTick => {
                println!("SingleTick");
            },
             m: PeriodicTick => {
                if (self.ticks == 3) {
                    self.timers.as_ref().unwrap().lock().unwrap().cancel(1);
                    println!("PeriodicTick cancelled");
                } else {
                    println!("PeriodicTick");
                    self.ticks = self.ticks + 1;
                }

            },
            _ => return false
        });

        true
    }
}