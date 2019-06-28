use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::message::Message;
use crate::actors::actor_context::ActorContext;
use crate::actors::actor::HandleResult;
use crate::actors::timers::{Timers, RealTimers, StubTimers};
use crate::actors::actor::PoisonPill;
use crate::common::tsafe::TSafe;
use std::time::Duration;
use std::collections::VecDeque;
use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard};
use match_downcast::*;

pub type StateResult<A, S, D> = Result<FsmAction<A, S, D>, Box<Any + Send>>;

pub enum FsmAction<A, S, D> {
    Goto(S),
    GotoUsing(S, D),
    Stay,
    StayUsing(D),
    Stop,
    Unhandled,
    Indirector(A)
}

pub struct StateTimeout<S> { state: S }


struct Handler<A, S, D> {
    state: S,
    timeout: Duration,
    f: Box<'static + Fn(&mut A, &Message, &mut ActorContext, &D) -> StateResult<A, S, D> + Send>
}

pub struct FsmWrapper<A, S, D> {
    inner: TSafe<Fsm<A, S, D>>
}

impl <A, S: 'static + PartialEq + std::fmt::Debug + Clone + Send, D> FsmWrapper<A, S, D> {
    pub fn register_handler<F>(&mut self, state: S, f: F, timeout: Duration)
        where F: 'static + Fn(&mut A, &Message, &mut ActorContext, &D) -> StateResult<A, S, D> + Send
    {
        self.inner.lock().unwrap().register_handler(state, f, timeout)
    }

    pub fn register_unhandled<F>(&mut self, f: F)
        where F: 'static + Fn(&mut A, &Message, &mut ActorContext, &S, &D) -> HandleResult + Send
    {
        self.inner.lock().unwrap().register_unhandled(f)
    }

    pub fn register_transition<F>(&mut self, f: F)
        where F: 'static + Fn(&mut A, &S, &S) + Send
    {
        self.inner.lock().unwrap().register_transition(f)
    }

    pub fn handle(&mut self, owner: &mut A, msg: Message, mut ctx: ActorContext) -> HandleResult {
        self.inner.lock().unwrap().handle(owner, msg, ctx)
    }

    pub fn initialize(&mut self, ctx: &ActorContext) {
        self.inner.lock().unwrap().initialize(ctx)
    }
}

impl <A, S, D> Clone for FsmWrapper<A, S, D> {
    fn clone(&self) -> Self {
        FsmWrapper {
            inner: self.inner.clone()
        }
    }
}

pub struct Fsm<A, S, D> {
    owner: Option<A>,
    handlers: Vec<Handler<A, S, D>>,
    unhandled: Option<Box<'static + Fn(&mut A, &Message, &mut ActorContext, &S, &D) -> HandleResult + Send>>,
    transition: Option<Box<'static + Fn(&mut A, &S, &S) + Send>>,
    timers: Timers,
    state: S,
    data: D
}


impl <A, S: 'static + PartialEq + std::fmt::Debug + Clone + Send, D> Fsm<A, S, D> {
    pub fn new(ctx: &ActorContext, i_state: S, i_data: D) -> FsmWrapper<A, S, D> {

        let inner = Fsm {
            owner: None,
            handlers: Vec::new(),
            unhandled: None,
            transition: None,
            timers: RealTimers::new(ctx.system.clone()),
            state: i_state,
            data: i_data
        };

        FsmWrapper { inner: tsafe!(inner) }
    }

    pub fn stub(i_state: S, i_data: D) -> FsmWrapper<A, S, D> {
        let inner = Fsm {
            owner: None,
            handlers: Vec::new(),
            unhandled: None,
            transition: None,
            timers: StubTimers::new(),
            state: i_state,
            data: i_data
        };

        FsmWrapper { inner: tsafe!(inner) }
    }

    pub fn register_handler<F>(&mut self, state: S, f: F, timeout: Duration)
        where F: 'static + Fn(&mut A, &Message, &mut ActorContext, &D) -> StateResult<A, S, D> + Send
    {
        self.handlers.push(Handler {
            state,
            timeout,
            f: Box::new(f)
        });
    }

    pub fn register_unhandled<F>(&mut self, f: F)
        where F: 'static + Fn(&mut A, &Message, &mut ActorContext, &S, &D) -> HandleResult + Send
    {
        self.unhandled = Some(Box::new(f))
    }

    pub fn register_transition<F>(&mut self, f: F)
        where F: 'static + Fn(&mut A, &S, &S) + Send
    {
        self.transition = Some(Box::new(f))
    }

    pub fn handle(&mut self, owner: &mut A, msg: Message, mut ctx: ActorContext) -> HandleResult {
        self.stop_state_timer();

        {
            let msg = msg.get();
            match_downcast_ref!(msg, {
                m: StateTimeout<S> => {
                    if m.state != self.state {
                        return Ok(true)
                    }
                },
                m: PoisonPill => {
                    return Ok(false)
                },
                _ => ()
            });
        }

        let handler = self.handlers.iter()
            .find(|v| v.state == self.state );

        if handler.is_some() {
            let h = handler.unwrap();
            let f = &h.f;
            let result: StateResult<A, S, D> = f(owner, &msg, &mut ctx, &self.data);

            if result.is_ok() {
                match result.ok().unwrap() {
                    FsmAction::Goto(state) => {
                        if self.transition.is_some() {
                            let f = self.transition.as_ref().unwrap();
                            f(owner, &self.state, &state)
                        }
                        self.state = state;
                        self.start_state_timer(&ctx);
                        Ok(true)
                    },
                    FsmAction::GotoUsing(state, data) => {
                        if self.transition.is_some() {
                            let f = self.transition.as_ref().unwrap();
                            f(owner, &self.state, &state)
                        }
                        self.state = state;
                        self.data = data;
                        self.start_state_timer(&ctx);
                        Ok(true)
                    },
                    FsmAction::Stay => {
                        self.start_state_timer(&ctx);
                        Ok(true)
                    },
                    FsmAction::StayUsing(data) => {
                        self.data = data;
                        self.start_state_timer(&ctx);
                        Ok(true)
                    },
                    FsmAction::Stop => {
                        ctx.system().stop(&mut ctx.self_.clone());
                        Ok(true)
                    },
                    FsmAction::Unhandled => {
                        self.start_state_timer(&ctx);

                        if self.unhandled.is_some() {
                            let f = self.unhandled.as_ref().unwrap();
                            f(owner, &msg, &mut ctx, &self.state, &self.data)
                        } else {
                            Ok(false)
                        }

                    }
                    _ => panic!("Indirector is used!")
                }

            } else {
                Err(result.err().unwrap())
            }
        } else {
            panic!("Unhandled fsm state '{:?}'", self.state);
        }
    }

    fn start_state_timer(&mut self, ctx: &ActorContext) {
        let handler = self.handlers.iter()
            .find(|v| v.state == self.state );

        if handler.is_some() {
            let h = handler.unwrap();

            self.timers.start_single(
                0,
                &ctx.self_,
                &ctx.self_,
                h.timeout.clone(),
                msg!(StateTimeout { state: self.state.clone() })
            )
        } else {
            panic!("Unhandled fsm state '{:?}'", self.state);
        }
    }

    fn stop_state_timer(&mut self) {
        self.timers.cancel(0)
    }

    pub fn initialize(&mut self, ctx: &ActorContext) {
        self.start_state_timer(ctx)
    }

    pub fn goto(state: S) -> StateResult<A, S, D> {
        Ok(FsmAction::Goto(state))
    }

    pub fn goto_using(state: S, data: D) -> StateResult<A, S, D> {
        Ok(FsmAction::GotoUsing(state, data))
    }

    pub fn stay() -> StateResult<A, S, D> {
        Ok(FsmAction::Stay)
    }

    pub fn stay_using(data: D) -> StateResult<A, S, D> {
        Ok(FsmAction::StayUsing(data))
    }

    pub fn stop() -> StateResult<A, S, D> {
        Ok(FsmAction::Stop)
    }

    pub fn unhandled() -> StateResult<A, S, D> {
        Ok(FsmAction::Unhandled)
    }
}

impl <A, S, D> Drop for Fsm<A, S, D> {
    fn drop(&mut self) {
        self.timers.cancel_all();
    }
}