use crate::actors::prelude::*;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props() -> Props {
    Props::new(tsafe!(Second::new()))
}

pub mod commands {
    pub struct GetResponse {
        pub data: u32
    }
}

pub mod responses {
    pub struct Response {
        pub data: u32
    }
}

pub struct Second {

}

impl Second {
    pub fn new() -> Second {
        Second {

        }
    }
}

impl Actor for Second {

    fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: commands::GetResponse => {
                let new_value = m.data + 100;
                ctx.sender.tell(msg!(responses::Response { data: new_value}), Some(&ctx.self_));
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}