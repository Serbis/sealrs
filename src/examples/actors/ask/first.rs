use crate::actors::prelude::*;
use super::second;
use std::sync::{Mutex, Arc};
use match_downcast::*;

pub fn props(second: &ActorRef) -> Props {
    Props::new(tsafe!(First::new(&second)))
}

pub mod commands {
    pub struct GetFlatResponse {}
    pub struct GetCascadeResponse { pub data: u32 }
}

pub mod responses {
    pub struct FlatResponse {}
    pub struct CascadeResponse { pub data: u32 }
}

pub struct First {
    second: ActorRef
}

impl First {
    pub fn new(second: &ActorRef) -> First {
        First {
            second: (*second).clone()
        }
    }
}



impl Actor for First {

    fn receive(&mut self, msg: Message, mut ctx: ActorContext) -> bool {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            _m: commands::GetFlatResponse => {
                ctx.sender.tell(msg!(responses::FlatResponse {}), Some(&ctx.self_));
            },


            m: commands::GetCascadeResponse => {
                let inter_data = m.data + 100;

                let mut sender = ctx.sender.clone();
                let mut self_ = ctx.self_.clone();

                // Here actor need send request to the second actor, and when he give the response
                // from him, will be created and sended response to the sender of the original
                // message. All actions of this operation will occurs asynchronous. Actor simply
                // create new ask request and forget about him. And will do some other work.
                // In this example he completes message processing.
                self.second.ask(&mut (*ctx.system()), msg!(second::commands::GetResponse { data: inter_data }) )
                    .on_complete(move |v| {
                        if v.is_ok() {
                            let resp = v.as_ref().ok().unwrap();
                            let resp = resp.get();
                            if let Some(m) = resp.downcast_ref::<second::responses::Response>() {
                                let inter = m.data;
                                sender.tell(msg!(responses::CascadeResponse {data: inter }), Some(&self_));
                            }
                        } else {
                            println!("Oops! Second does not respond!")
                        }
                    });
            },


            _ => return false
        });

        true
    }
}