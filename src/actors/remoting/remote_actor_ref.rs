//! Actor reference which points to an actor in a remote actor system.

use crate::common::tsafe::TSafe;
use crate::actors::actor_cell::ActorCell;
use crate::actors::abstract_actor_ref::{AbstractActorRef, ActorRef, AskTimeoutError};
use crate::actors::actor_path::ActorPath;
use crate::actors::message::Message;
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::remoting::net_controller::NetController;
use crate::actors::props::Props;
use crate::actors::ask_actor::AskActor;
use crate::futures::future::WrappedFuture;
use crate::futures::promise::Promise;
use crate::futures::completable_promise::CompletablePromise;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::any::Any;
use std::time::Duration;
use std::sync::{Arc, Mutex};

pub struct RemoteActorRef {
    /// Remote connection id
    pub rcid: u32,

    /// Remote ActorRef id
    pub rarid: u32,

    pub path: TSafe<ActorPath>,

    /// Network controller instance
    pub net_controller: TSafe<NetController + Send>
}

impl RemoteActorRef {

    pub fn new(rcid: u32, rarid: u32, path: TSafe<ActorPath>, net_controller: TSafe<NetController + Send>) -> RemoteActorRef {
        RemoteActorRef {
            rcid,
            rarid,
            path,
            net_controller
        }
    }

    fn inner_clone(self: &Self) -> Box<RemoteActorRef> {
        Box::new(RemoteActorRef {
            rcid: self.rcid,
            rarid: self.rarid,
            path: self.path.clone(),
            net_controller: self.net_controller.clone()
        })
    }
}

impl AbstractActorRef for RemoteActorRef {

    fn tell(self: &mut Self, msg: Message, rself: Option<&ActorRef>) {
        let mut controller = self.net_controller.lock().unwrap();
        let sender = rself.map_or(None, |v| Some((*v).clone()));
        controller.send_msg(msg, self.rcid, self.rarid, sender, self.clone());
    }

    fn ask(&mut self, factory: &mut AbstractActorSystem, msg: Message) -> WrappedFuture<Message, AskTimeoutError> {
        self.ask_timeout(factory, Duration::from_secs(3), msg)
    }

    fn ask_timeout(&mut self, factory: &mut AbstractActorSystem, timeout: Duration, msg: Message) -> WrappedFuture<Message, AskTimeoutError> {
        let p: CompletablePromise<Message, AskTimeoutError>
        = CompletablePromise::new();
        let f = p.future();

        let ask_actor = factory.actor_of(Props::new(tsafe!(AskActor::new(p, timeout))), None);
        self.tell(msg, Some(&ask_actor));

        f
    }

    fn path(&self) -> ActorPath {
        self.path.lock().unwrap().clone()
    }

    fn cell(self: &mut Self) -> TSafe<ActorCell> {
        panic!("Cannot get cell on the remote actor reference")
    }

    fn clone(self: &Self) -> ActorRef {
        self.inner_clone()
    }

    fn as_any(self: &Self) -> Box<Any> {
        Box::new(self.inner_clone())
    }
}

impl fmt::Display for RemoteActorRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RemoteActorRef ({})", self.path.lock().unwrap())
    }
}

impl PartialEq for RemoteActorRef {
    fn eq(&self, other: &Self) -> bool {
        *self.path.lock().unwrap() == *other.path.lock().unwrap()
    }
}

impl Eq for RemoteActorRef {}

impl Hash for RemoteActorRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.lock().unwrap().hash(state);
    }
}