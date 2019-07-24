//! Network controller trait.
//!
//! This trait is used with partial controllers for realize various identical action.

use crate::actors::message::Message;
use crate::actors::abstract_actor_ref::ActorRef;

pub trait NetController {
    fn send_msg(&mut self, msg: Message, rcid: u32, rarid: u32, sender: Option<ActorRef>, far: ActorRef);
}