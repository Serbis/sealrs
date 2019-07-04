//! Message processing context
//!
//! This object will constructs for each new message received by an actor
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::unbound_mailbox::UnboundMailbox;
use crate::actors::actor_path::ActorPath;
use crate::actors::local_actor_ref::LocalActorRef;
use crate::actors::actor_cell::ActorCell;
use crate::actors::pinned_dispatcher::PinnedDispatcher;
use crate::actors::props::Props;
use crate::common::tsafe::TSafe;
use crate::actors::dispatcher::Dispatcher;
use std::sync::{Mutex, Arc, MutexGuard};

//pub struct WrappedCell {
//    refed: Option<&ActorCell>,
//    safe: Option<>
//}

pub struct ActorContext {

    /// Who send the current message
    pub sender: ActorRef,

    /// Own actor reference
    pub self_: ActorRef,

    /// Actor system where actor is work
    pub system: TSafe<AbstractActorSystem + Send>,

    /// Link to the actor cell
    pub cell: TSafe<ActorCell>
}

impl ActorContext {
    pub fn new(sender: ActorRef,
               self_: ActorRef,
               system: TSafe<AbstractActorSystem + Send>,
               cell: TSafe<ActorCell>) -> ActorContext
    {
        ActorContext {
            sender,
            self_,
            system,
            cell
        }
    }

    pub fn system(&self) -> MutexGuard<AbstractActorSystem + Send + 'static> {
        self.system.lock().unwrap()
    }
}

impl ActorRefFactory for ActorContext {
    fn actor_of(self: &mut Self, props: Props, name: Option<&str>) -> ActorRef {
        let mailbox = tsafe!(UnboundMailbox::new());

        let mut aname: String;

        if name.is_some() {
            aname = name.unwrap().to_string();
        } else {
            aname = self.system().get_nid();
        }

        let path = tsafe!(ActorPath::new(&aname, Some(self.cell.lock().unwrap().path.clone())));

        let dispatcher: TSafe<Dispatcher + Send> = {
            match &(props.dispatcher)[..] {
                "default" => {
                    let dispatchers = self.system().get_dispatchers();
                    let d = dispatchers.lock().unwrap();
                    let d = d.get(&String::from("default"));
                    let d = d.as_ref().unwrap();
                    (*d).clone()
                },
                "pinned" => tsafe!(PinnedDispatcher::new()),
                other => {
                    let dispatchers = self.system().get_dispatchers();
                    let dispatchers = dispatchers.lock().unwrap();
                    let dispatcher = dispatchers.get(&String::from(other));

                    match dispatcher {
                        Some(d) => d.clone(),
                        None => panic!("Dispatcher with name '{}' does not registered", other)
                    }
                }
            }
        };


        let cell = ActorCell::new(
            self.system.clone(),
            path.clone(),
            props.actor,
            0,
            dispatcher,
            mailbox,
            Some(self.cell.clone()),
            props.supervision_strategy);
        let boxed_cell = tsafe!(cell);

        {
            let mut root = self.cell.lock().unwrap();
            let exists = root.childs.get(&aname);
            if exists.is_some() {
                panic!("Unable to create actor with existed name '{}'", aname)
            }
            root.childs.insert(aname, boxed_cell.clone());
        }

        let f = boxed_cell.lock().unwrap().start(boxed_cell.clone());
        f();

        Box::new(LocalActorRef::new(boxed_cell, path))
    }

    fn actor_select(&mut self, path: &str) -> Vec<ActorRef> {
        self.system().actor_select(path)
    }

    fn stop(self: &mut Self, aref: &mut ActorRef) {
        self.system().stop(aref);
    }

    fn dead_letters(self: &mut Self) -> ActorRef {
        self.system().dead_letters()
    }

    fn watch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        self.system().watch(watcher, observed)
    }

    fn unwatch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        self.system().unwatch(watcher, observed)
    }
}