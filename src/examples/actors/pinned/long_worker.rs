use crate::actors::prelude::*;
use match_downcast::*;
use std::time::{Duration};
use std::thread;


pub struct DoLongWork {
    pub time: Duration,
}


pub struct LongWorker {
    marker: String
}

impl LongWorker {
    pub fn new(marker: &str) -> LongWorker {
        LongWorker {
            marker: String::from(marker)
        }
    }
}

impl Actor for LongWorker {

    fn receive(self: &mut Self, msg: Message, _ctx: ActorContext) -> bool {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: DoLongWork => {
                thread::sleep(m.time);

                println!("LongWorker - {} / ended doing some work within {} milliseconds",
                    self.marker, m.time.as_millis());
            },
            _ => return false
        });

        true
    }

}