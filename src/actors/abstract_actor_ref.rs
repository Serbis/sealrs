use crate::actors::actor_path::ActorPath;
use crate::actors::actor_cell::ActorCell;
use crate::common::tsafe::TSafe;
use std::any::Any;
use std::fmt;

pub type ActorRef = Box<AbstractActorRef + Send>;

trait DisplayAndCopy {}



pub trait AbstractActorRef: fmt::Display {
    fn tell(self: &mut Self, msg: Box<Any + Send + 'static>, rself: Option<&ActorRef>);
    fn path(&self) -> ActorPath;
    fn cell(self: &mut Self) -> TSafe<ActorCell>;
    fn clone(self: &Self) -> ActorRef;
    fn as_any(self: &Self) -> Box<Any>;
}


//impl PartialEq for AbstractActorRef {
//    fn eq(&self, other: &Self) -> bool;
//}

//impl Eq for AbstractActorRef {}
//
//impl Hash for AbstractActorRef {
//    fn hash<H: Hasher>(&self, state: &mut H) {
//        self.path.lock().unwrap().hash(state);
//    }
//}

