use crate::actors::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod commands {

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GetData { pub c: u32 }
}

pub mod responses {

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Data { pub value: String }
}

pub struct Storage {}

impl Storage {
    pub fn props() -> Props { Props::new(tsafe!(Storage::new()))}

    pub fn new() -> Storage {
        Storage {}
    }
}

impl Actor for Storage {
    fn receive(&mut self, msg: Message, mut ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: commands::GetData => {
                println!("Server -> message={:?}", m);

                // Attention! Never do such thing in a real code. This line was add here only for
                // reduce code complexity. In a real code use Timers for such actions.
                thread::sleep(Duration::from_secs(1));

                // Don't do that in the real code. This call interact with RemoteActorRef, which
                // may cause to block. For use ths code as is, you need to run this actor on the
                // pinned dispatcher, otherwise you must run this line in the async future.
                ctx.sender.tell(msg!(responses::Data { value: format!("Data-{}", m.c) }), Some(&ctx.self_));
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}