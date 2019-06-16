//! Reference to instance of some actor. It is an interface for interacting with the internal
//! logic of the actor cell.

use crate::common::tsafe::TSafe;
use crate::actors::actor_cell::ActorCell;
use crate::actors::abstract_actor_ref::{AbstractActorRef, ActorRef, AskTimeoutError};
use crate::actors::actor_path::ActorPath;
use crate::actors::message::Message;
use crate::actors::abstract_actor_system::AbstractActorSystem;
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

pub struct LocalActorRef {
    pub cell: TSafe<ActorCell>,
    pub path: TSafe<ActorPath>
}

impl LocalActorRef {

    /// Creates a new reference. This method should never be invoked by application code. This
    /// constructor is used by internal API. Direct use from the user code is prohibited.
    pub fn new(cell: TSafe<ActorCell>, path: TSafe<ActorPath>) -> LocalActorRef {
        LocalActorRef {
            cell,
            path
        }
    }

    fn inner_clone(self: &Self) -> Box<LocalActorRef> {
        Box::new(LocalActorRef {
            cell: self.cell.clone(),
            path: self.path.clone(),
        })
    }
}

impl AbstractActorRef for LocalActorRef {

    /// Send message to the actor behind the reference. In the first argument passed message for
     /// send, and in the second argument specified sender reference. This reference will be injected
     /// to ctx.sender field the actor context object. If sender was does not specified, ctx.sender
     /// will be filled with the deadLetter actor reference. Setting up None as sender reference is
     /// useful in case, when tell operation is
     /// called from outside of the actor system.
     ///
     /// # Examples
     ///
     /// ```
     ///
     /// ```
    fn tell(self: &mut Self, msg: Message, rself: Option<&ActorRef>) {
        let cell_cloned = self.cell.clone();
        let path_cloned = self.path.clone();
        let toref = Box::new(LocalActorRef::new(cell_cloned, path_cloned));
        let mut cell = self.cell.lock().unwrap();
        cell.send(&self.cell, msg, rself.map_or(None, |v| Some((*v).clone())), toref)
    }

    /// Call ask_timeout with default timeout
    fn ask(&mut self, factory: &mut AbstractActorSystem, msg: Message) -> WrappedFuture<Message, AskTimeoutError> {
        self.ask_timeout(factory, Duration::from_secs(3), msg)
    }

    /// Sends a message to the target actor and expects that he respond to that message. Expectation
    /// represent as future, which will be completed with received message or will be failed with
    /// AskTimeoutError, if target actor will not respond with specified timeout.
    ///
    /// # Example
    ///
    /// ```
    /// first.ask(&mut (*ctx.system()), Duration::from_secs(3), msg!(SomeMsg {}))
    ///     .on_complete(|v| {
    ///         // Do something with result (or error)
    ///     });
    /// ```
    ///
    fn ask_timeout(&mut self, factory: &mut AbstractActorSystem, timeout: Duration, msg: Message) -> WrappedFuture<Message, AskTimeoutError> {
        let p: CompletablePromise<Message, AskTimeoutError>
            = CompletablePromise::new();
        let f = p.future();

        let ask_actor = factory.actor_of(Props::new(tsafe!(AskActor::new(p, timeout))), None);
        self.tell(msg, Some(&ask_actor));

        f
    }

    /// Return copy of the actor path object
    fn path(&self) -> ActorPath {
        self.path.lock().unwrap().clone()
    }

    fn cell(self: &mut Self) -> TSafe<ActorCell> {
        self.cell.clone()
    }

    fn clone(self: &Self) -> ActorRef {
        self.inner_clone()
    }

    fn as_any(self: &Self) -> Box<Any> {
        Box::new(self.inner_clone())
    }
}

impl fmt::Display for LocalActorRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ActorRef ({})", self.path.lock().unwrap())
    }
}

/*impl Clone for LocalActorRef {
    fn clone(&self) -> LocalActorRef {
        LocalActorRef {
            cell: self.cell.clone(),
            path: self.path.clone()
        }
    }
}*/

impl PartialEq for LocalActorRef {
    fn eq(&self, other: &Self) -> bool {
        *self.path.lock().unwrap() == *other.path.lock().unwrap()
    }
}

impl Eq for LocalActorRef {}

impl Hash for LocalActorRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.lock().unwrap().hash(state);
    }
}