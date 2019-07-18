use crate::actors::prelude::*;
use super::remote_messages::storage;
use std::sync::{Arc, Mutex};

pub struct Requestor {
    storage: ActorRef,
    counter: u32
}

impl Requestor {
    pub fn props(storage: ActorRef) -> Props { Props::new(tsafe!(Requestor::new(storage)))}

    pub fn new(storage: ActorRef) -> Requestor {
        Requestor {
            storage,
            counter: 0
        }
    }
}

impl Actor for Requestor {

    fn pre_start(&mut self, ctx: ActorContext) {
        self.storage.tell(msg!(storage::commands::GetData {c: self.counter }), Some(&ctx.self_));
        self.counter = self.counter + 1;
    }

    fn receive(&mut self, msg: Message, mut ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: storage::responses::Data => {
                println!("Client -> message={:?}", m);

                // Don't do that in the real code. This call interact with RemoteActorRef, which
                // may cause to block. For use ths code as is, you need to run this actor on the
                // pinned dispatcher, otherwise you must run this line in the async future.
                self.storage.tell(msg!(storage::commands::GetData {c: self.counter }), Some(&ctx.self_));

                self.counter = self.counter + 1;
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}