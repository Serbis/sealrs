/// Internal actor of the Ask request
///
/// This actor favors as middlemen between target actor and future returned by the ask call. He
/// receive response from a target actor and completes promise with him. Also this actor start
/// the timer, which determines timeout state. If it occurs before any message will be received,
/// promise will be failed with AskTimeoutError.
///
use crate::common::tsafe::TSafe;
use crate::actors::actor::{Actor, HandleResult};
use crate::actors::actor::PoisonPill;
use crate::actors::actor_context::ActorContext;
use crate::actors::timers::{Timers, RealTimers};
use crate::actors::message::Message;
use crate::actors::abstract_actor_ref::AskTimeoutError;
use crate::futures::promise::Promise;
use crate::futures::completable_promise::CompletablePromise;
use std::sync::{Mutex, Arc};
use match_downcast::*;
use std::time::Duration;

struct Timeout {}

pub struct AskActor {
    timers: Option<TSafe<Timers>>,
    p: CompletablePromise<Message, AskTimeoutError>,
    timeout: Duration
}

impl AskActor {
    pub fn new(p: CompletablePromise<Message, AskTimeoutError>, timeout: Duration) -> AskActor {
        AskActor {
            timers: None,
            p,
            timeout
        }
    }
}

impl Actor for AskActor {

    fn pre_start(self: &mut Self, ctx: ActorContext) {
        let mut timers = RealTimers::new(ctx.system.clone());

        timers.start_single(
            0,
            &ctx.self_,
            &ctx.self_,
            self.timeout,
            msg!(Timeout {}));

        self.timers = Some(tsafe!(timers));
    }


    fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> HandleResult {
        let tp = {
            match_downcast_ref!(msg.get(), {
                _m: Timeout => {
                    0
                },
                 _m: PoisonPill => {
                    1
                },
                _ => {
                    2
                }
            })
        };




        if tp == 0 {
            self.p.failure(AskTimeoutError {});
            self.timers.as_ref().unwrap().lock().unwrap().cancel_all();
            ctx.system.lock().unwrap().stop(&mut ctx.self_);
            Ok(true)
        } else if tp == 1 {
            Ok(false)
        } else {
            self.p.success(msg.clone());
            self.timers.as_ref().unwrap().lock().unwrap().cancel_all();
            ctx.system.lock().unwrap().stop(&mut ctx.self_);
            Ok(true)
        }
    }
}