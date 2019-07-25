//! User side mailbox for actors with non-linear behaviors

use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::message::Message;
use crate::actors::actor_context::ActorContext;
use std::collections::VecDeque;
use std::mem;

pub type Stash = Box<AbstractStash + Send>;

pub trait AbstractStash {

    /// Put message to the queue
    fn stash(&mut self, msg: &Message, ctx: &ActorContext);

    /// Flush all queue (each message will be resend to the stash owner actor)
    fn unstash_all(&mut self);

    /// Stops self
    fn stop(&mut self);
}

struct StashEnvelope {
    sender: ActorRef,
    message: Message
}

pub struct RealStash {
    self_: Option<ActorRef>,
    queue: VecDeque<StashEnvelope>
}


impl RealStash {
    pub fn new(ctx: &ActorContext) -> Stash {
        let s = RealStash {
            self_: Some(ctx.self_.clone()),
            queue: VecDeque::new()
        };

        Box::new(s)
    }
}

/// Normal version of the stash
impl AbstractStash for RealStash {
    fn stash(&mut self, msg: &Message, ctx: &ActorContext) {
        let envelope = StashEnvelope {
            sender: ctx.sender.clone(),
            message: msg.clone()
        };

        self.queue.push_back(envelope);
    }

    fn unstash_all(&mut self) {
        let mut s = mem::replace(&mut self.self_, None).unwrap();
        while self.queue.len() > 0 {
            let envelope = self.queue.pop_front().unwrap();
            s.tell(envelope.message, Some(&envelope.sender));
        }
        self.self_ = Some(s);
    }

    fn stop(&mut self) {
        self.self_ = None;
    }
}

/// Stub object due to prevent Option<Stash> in an actor state
pub struct StubStash {}

impl StubStash {
    pub fn new() -> Stash {
        Box::new(StubStash {})
    }
}

impl AbstractStash for StubStash {
    fn stash(&mut self, _msg: &Message, _ctx: &ActorContext) {
        unimplemented!()
    }

    fn unstash_all(&mut self) {
        unimplemented!()
    }

    fn stop(&mut self) { unimplemented!() }
}