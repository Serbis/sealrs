//! Mailbox interface
//!
//! This interface defines functions, which must be implemented by the specific mailbox realization.
//!
use crate::actors::envelope::Envelope;
use crate::actors::abstract_actor_ref::AbstractActorRef;
//TODO нужно переделать mailbox, так как блокироваться должно очередь внутри него а не сам объект. При текущем варинте невозможно реализация блокирующих ящиков.
pub trait Mailbox {

    /// Set scheduled marker. This marker indicates than messages from this mailbox was planned for
    /// execution by dispatcher
    fn set_planned(self: &mut Self, planned: bool);

    /// Return scheduled marker
    fn is_planned(self: &Self) -> bool;

    /// Enqueue new message to the mailbox
    fn enqueue(self: &mut Self, envelope: Envelope);

    /// Dequeue new message to the mailbox
    fn dequeue(self: &mut Self) -> Envelope;

    /// Clean mailbox, This function receive a reference to the owner actor, and reference to
    /// DeadLetter for dropping messages.
    fn clean_up(self: &mut Self, sender: Box<AbstractActorRef>, dead_letters: Box<AbstractActorRef>);

    /// Checks messages existing in the mailbox
    fn has_messages(self: &Self) -> bool;
}