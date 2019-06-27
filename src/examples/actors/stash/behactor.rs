use crate::actors::prelude::*;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props() -> Props {
    Props::new(tsafe!(Behactor::new()))
}

pub struct MessageA {}
pub struct MessageB {}
pub struct MessageC {}
pub struct MessageD {}

pub struct Behactor {
    stash: Stash,
    behavior: u32
}

impl Behactor {
    pub fn new() -> Behactor {
        Behactor {
            stash: StubStash::new(),
            behavior: 0
        }
    }
}

impl Actor for Behactor {

    fn pre_start(self: &mut Self, ctx: ActorContext) {
        self.stash = RealStash::new(&ctx)
    }

    fn receive(self: &mut Self, msg: Message, ctx: ActorContext) -> HandleResult {
        let message = msg.get();

        match self.behavior {
            0 => {
                match_downcast_ref!(message, {
                    _m: MessageA => {
                        println!("MessageA - switch to behavior 1");
                        self.behavior = 1;

                    },
                    _m: MessageB => {
                        println!("MessageB");

                    },
                    _m: MessageC => {
                        println!("MessageC");

                    },

                    _ => return Ok(false)
                });
            },
            _ => {
                match_downcast_ref!(message, {
                    _m: MessageD => {
                        println!("MessageD - switch to behavior 0");
                        self.behavior = 0;
                        self.stash.unstash_all();

                    },
                    _ => {
                        println!("Stashed message not from this behavior");
                        self.stash.stash(&msg, &ctx);
                    }
                });
            }
        }


        Ok(true)
    }
}