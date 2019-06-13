use crate::actors::actor_path::ActorPath;
use crate::actors::actor_cell::ActorCell;
use crate::actors::message::Message;
use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::futures::future::WrappedFuture;
use crate::common::tsafe::TSafe;
use std::any::Any;
use std::fmt;
use std::time::Duration;

pub type ActorRef = Box<AbstractActorRef + Send>;

#[derive(Clone)]
pub struct AskTimeoutError {}

pub trait AbstractActorRef: fmt::Display {
    fn tell(self: &mut Self, msg: Message, rself: Option<&ActorRef>);
    fn ask(&mut self, factory: &mut AbstractActorSystem, msg: Message) -> WrappedFuture<Message, AskTimeoutError>;
    fn ask_timeout(&mut self, factory: &mut AbstractActorSystem,  timeout: Duration, msg: Message,) -> WrappedFuture<Message, AskTimeoutError>;
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

