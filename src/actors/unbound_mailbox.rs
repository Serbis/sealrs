//! Predefined mailbox without queue size restrictions
//!
//! Is is the default actor mailbox. When using it, you must be careful, because if your system will
//! can't handle the load, this mailbox will be the main point of failure. He simply will use all
//! existed RAM.
//!
use crate::actors::mailbox::Mailbox;
use crate::actors::envelope::Envelope;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::message::Message;
use std::collections::vec_deque::VecDeque;

use std::sync::{Arc, Mutex};

pub struct UnboundMailbox {
    is_planned: bool,
    queue: VecDeque<Envelope>
}

impl UnboundMailbox {
    pub fn new() -> UnboundMailbox {
        UnboundMailbox {
            is_planned: false,
            queue: VecDeque::new()
        }
    }
}

impl Mailbox for UnboundMailbox {

    fn set_planned(self: &mut Self, planned: bool) {
        self.is_planned = planned;
    }

    fn is_planned(self: &Self) -> bool {
        self.is_planned
    }

    fn enqueue(self: &mut Self, envelope: Envelope) {
        self.queue.push_back(envelope);
    }

    fn dequeue(self: &mut Self) -> Envelope {
        self.queue.pop_front().unwrap()
    }

    fn has_messages(self: &Self) -> bool {
        self.queue.len() > 0
    }

    /// Drops all messages to the DeadLetter
    fn clean_up(self: &mut Self, sender: ActorRef, dead_letters: ActorRef) {
        let mut dead_letters = dead_letters;

        while true {
            if self.queue.len() > 0 {
                dead_letters.tell(msg!(self.queue.pop_front().unwrap()), Some(&sender)); //(self.queue.pop_front().unwrap()
            } else {
                break;
            }
        }
    }
}