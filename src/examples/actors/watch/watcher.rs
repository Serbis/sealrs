use crate::actors::actor::Actor;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::message::Message;
use crate::actors::actor_context::ActorContext;
use crate::actors::watcher::events::Terminated;
use crate::actors::props::Props;
use match_downcast::*;
use std::sync::{Mutex, Arc};

pub fn props(target: ActorRef) -> Props {
    Props::new(tsafe!(Watcher::new(target)))
}


pub struct Watcher {
    target: ActorRef
}

impl Watcher {
    pub fn new(target: ActorRef) -> Watcher {
        Watcher {
            target
        }
    }
}

impl Actor for Watcher {

    fn pre_start(&mut self, ctx: ActorContext) {

        // Registers target actor as watched
        ctx.system.lock().unwrap().watch(&ctx.self_, &self.target);
    }

    fn post_stop(&mut self, ctx: ActorContext) {

        // Unregisters target actor as watched
        ctx.system.lock().unwrap().unwatch(&ctx.self_, &self.target);
    }

    fn receive(self: &mut Self, msg: &Message, ctx: ActorContext) -> bool {
        match_downcast_ref!(msg, {
            _m: Terminated => {
                // Sender of this message is the target actor. It indicates that the target actor was
                // stopped
                if ctx.sender.path() == self.target.path() {
                    println!("Target actor '{}' was terminated", ctx.sender);
                }
            },
            _ => return false
        });

        true
    }
}