use crate::actors::actor_path::ActorPath;
use crate::actors::actor_cell::ActorCell;
use crate::common::tsafe::TSafe;
use std::any::Any;
use std::fmt;

pub type ActorRef = Box<AbstractActorRef + Send>;

trait DisplayAndCopy {}



pub trait AbstractActorRef: fmt::Display {
    fn tell(self: &mut Self, msg: Box<Any + Send + 'static>, rself: Option<ActorRef>);
    fn path(self: &mut Self) -> ActorPath;
    fn cell(self: &mut Self) -> TSafe<ActorCell>;
    fn clone(self: &Self) -> ActorRef;
    fn as_any(self: &Self) -> Box<Any>;
}

/*impl Clone for Box<AbstractActorRef> {
    fn clone(&self) -> Self {
        self.clone_()
    }
}*/