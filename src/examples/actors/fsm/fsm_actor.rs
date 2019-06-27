use crate::actors::prelude::*;
use crate::common::tsafe::TSafe;
use std::sync::{Mutex, Arc};
use std::time::Duration;
use match_downcast::*;
use std::any::Any;

pub fn props() -> Props {
    Props::new(tsafe!(FsmActor::new()))
}

pub struct MessageA { pub v: u32 }
pub struct MessageB {}
pub struct MessageC {}
pub struct MessageD {}
pub struct MessageOther {}

#[derive(PartialEq, Debug, Clone)]
pub enum State {
    State0,
    State1
}

#[derive(Debug)]
pub enum Data {
    Data0,
    Data1(u32)
}

pub struct FsmActor {
    fsm: FsmWrapper<FsmActor, State, Data>
}


impl FsmActor {
    pub fn new() -> FsmActor {
        FsmActor {
            fsm: Fsm::stub(State::State0, Data::Data0)
        }
    }

    pub fn state0(&mut self, msg: &Message, ctx: &ActorContext, data: &Data) -> StateResult<Self, State, Data> {
        let msg = msg.get();

        match_downcast_ref!(msg, {
            m: MessageA => {
                println!("Handled 'MessageA' in state 'State0'");
                Fsm::goto_using(State::State1, Data::Data1(m.v))
            },
            _ => Fsm::unhandled()
        })
    }

    pub fn state1(&mut self, msg: &Message, ctx: &ActorContext, data: &Data) -> StateResult<Self, State, Data> {
        let msg = msg.get();

        match data {
            Data::Data1(v) => {
                match_downcast_ref!(msg, {
                    m: MessageB => {
                        println!("Handled 'MessageB' in state 'State1' and data Data1({})", v);
                        Fsm::stay()
                    },
                    m: StateTimeout<State> => {
                        println!("Handled StateTimeout in state 'State1'");
                        Fsm::stay()
                    },
                    _ => Fsm::unhandled()
                })
            },
            _ => Fsm::unhandled()

        }
    }

    pub fn unhandled(&mut self, msg: &Message, ctx: &ActorContext, state: &State, data: &Data) -> HandleResult {
        println!("Unhandled message in state '{:?}' with data '{:?}'", state, data);
        Ok(true)
    }

    pub fn transition(&mut self, from: &State, to: &State) {
        println!("State transition from '{:?}' to '{:?}'", from, to);
    }

}

impl Actor for FsmActor {

    fn pre_start(&mut self, ctx: ActorContext) {
        let mut fsm = Fsm::new(&ctx, State::State0, Data::Data0);

        fsm.register_handler(State::State0, Self::state0, Duration::from_secs(5));
        fsm.register_handler(State::State1, Self::state1, Duration::from_secs(5));
        fsm.register_unhandled(Self::unhandled);
        fsm.register_transition(Self::transition);
        fsm.initialize(&ctx);

        self.fsm = fsm;
    }

    fn receive(&mut self, msg: Message, ctx: ActorContext) -> HandleResult {
        self.fsm.clone().handle(self, msg, ctx)
    }
}