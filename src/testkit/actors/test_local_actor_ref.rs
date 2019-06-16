//! Test variant of the LocalActorRef.
//!
//! This object is fully mirror of the original actor system version, but additionally extends he
//! with various  methods for tests performing. The source code contains special comments which
//! marks code which was fully cloned from the original actor system. All codes outside of this
//! blocks, is code of a tests extensions.

use crate::common::tsafe::TSafe;
use crate::actors::actor_cell::ActorCell;
use crate::actors::abstract_actor_ref::{ActorRef, AbstractActorRef, AskTimeoutError};
use crate::actors::actor_path::ActorPath;
use crate::actors::actor::Actor;
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::props::Props;
use crate::actors::ask_actor::AskActor;
use crate::actors::message::Message;
use crate::futures::future::WrappedFuture;
use crate::futures::promise::Promise;
use crate::futures::completable_promise::CompletablePromise;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::any::Any;
use std::time::Duration;
use std::sync::{Arc, Mutex};

pub struct TestLocalActorRef {

    // ------- mirror ---------
    pub cell: TSafe<ActorCell>,
    pub path: TSafe<ActorPath>,
    // --------- end ----------

    /// Original actor object on which this reference links
    pub actor: TSafe<Actor + Send>
}


impl TestLocalActorRef {

    /// Identical to original
    pub fn new(cell: TSafe<ActorCell>, path: TSafe<ActorPath>) -> TestLocalActorRef {
        let actor = cell.clone().lock().unwrap().actor.clone();
        TestLocalActorRef {
            cell,
            path,
            actor
        }
    }

    fn inner_clone(self: &Self) -> Box<TestLocalActorRef> {
        Box::new(TestLocalActorRef {
            cell: self.cell.clone(),
            path: self.path.clone(),
            actor: self.actor.clone()
        })
    }
}

impl AbstractActorRef for TestLocalActorRef {

    /// Identical to original
    fn tell(self: &mut Self, msg: Message, rself: Option<&ActorRef>) {
        // ------- mirror ---------
        let cell_cloned = self.cell.clone();
        let path_cloned = self.path.clone();
        let toref = Box::new(TestLocalActorRef::new(cell_cloned, path_cloned));
        let mut cell = self.cell.lock().unwrap();
        cell.send(&self.cell, msg, rself.map_or(None, |v| Some((*v).clone())), toref);
        // --------- end ----------
    }

    /// Identical to original
    fn ask(&mut self, factory: &mut AbstractActorSystem, msg: Message) -> WrappedFuture<Message, AskTimeoutError> {
        self.ask_timeout(factory, Duration::from_millis(500), msg)
    }

    /// Identical to original
    fn ask_timeout(&mut self, factory: &mut AbstractActorSystem, timeout: Duration, msg: Message) -> WrappedFuture<Message, AskTimeoutError> {
        let p: CompletablePromise<Message, AskTimeoutError>
        = CompletablePromise::new();
        let f = p.future();

        let ask_actor = factory.actor_of(Props::new(tsafe!(AskActor::new(p, timeout))), None);
        self.tell(msg, Some(&ask_actor));

        f
    }

    /// Identical to original
    fn path(&self) -> ActorPath {
        // ------- mirror ---------
        self.path.lock().unwrap().clone()
        // --------- end ----------
    }

    /// Identical to original
    fn cell(self: &mut Self) -> TSafe<ActorCell> {
        // ------- mirror ---------
        self.cell.clone()
        // --------- end ----------
    }

    /// Identical to original
    fn clone(self: &Self) -> ActorRef {
        self.inner_clone()
    }

    /// Identical to original
    fn as_any(self: &Self) -> Box<Any> {
        Box::new(self.inner_clone())
    }
}

impl fmt::Display for TestLocalActorRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TestActorRef ({})", self.path.lock().unwrap())
    }
}

/*impl Clone for TestLocalActorRef {
    fn clone(&self) -> TestLocalActorRef {
        TestLocalActorRef {
            cell: self.cell.clone(),
            path: self.path.clone(),
            actor: self.actor.clone()
        }
    }
}*/

impl PartialEq for TestLocalActorRef {
    fn eq(&self, other: &Self) -> bool {
        *self.path.lock().unwrap() == *other.path.lock().unwrap()
    }
}

impl Eq for TestLocalActorRef {}

impl Hash for TestLocalActorRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.lock().unwrap().hash(state);
    }
}