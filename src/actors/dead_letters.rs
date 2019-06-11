//! Crematorium for undelivered messages
//!
//! All messages enqueued to this mailbox will be logged and destroyed.
//!
use crate::actors::mailbox::Mailbox;
use crate::actors::envelope::Envelope;
use crate::actors::abstract_actor_ref::ActorRef;

pub struct DeadLetters {
    is_planned: bool
}

impl DeadLetters {
    pub fn new() -> DeadLetters {
        DeadLetters {
            is_planned: true // It's always planned to prevent it's execution
        }
    }
}

impl Mailbox for DeadLetters {

    /// He is always planned. This is needed for prevent scheduling of the mailbox
    fn set_planned(self: &mut Self, _planned: bool) {
        self.is_planned = true;
    }

    /// I say again, he is always planned!
    fn is_planned(self: &Self) -> bool {
        true
    }

    /// Constructs a beautiful message and print it
    fn enqueue(self: &mut Self, envelope: Envelope) {
        let mut actor_name = "outside".to_string();

        if envelope.sender.is_some() {
            let sender =  &mut envelope.sender.unwrap();
            let sender_path = {
                sender.path().name.clone()
            };
            if sender_path != "deadLetters" {
                actor_name = sender.to_string();
            }
            //
        }

        println!("DeadLetter receive some message ... from '{}' to '{}'", actor_name, envelope.receiver) //
    }

    /// Oops! This mailbox does not contains the queue
    fn dequeue(self: &mut Self) -> Envelope {
        panic!("Try to dequeue deadLetter mailbox")
    }

    /// He never store messages
    fn has_messages(self: &Self) -> bool {
        false
    }

    /// Do nothing
    fn clean_up(self: &mut Self, _sender: ActorRef, _dead_letters: ActorRef) {}
}