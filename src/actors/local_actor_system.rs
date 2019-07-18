//! Actor system is a container for actors runtime. She is used for actor creation,  stop and other
//! operations

use crate::common::tsafe::TSafe;
use crate::actors::props::Props;
use crate::actors::actor_path::ActorPath;
use crate::actors::actor_cell::ActorCell;
use crate::actors::actor::PoisonPill;
use crate::actors::default_dispatcher::DefaultDispatcher;
use crate::actors::pinned_dispatcher::PinnedDispatcher;
use crate::actors::dispatcher::Dispatcher;
use crate::actors::dead_letters::DeadLetters;
use crate::actors::synthetic_actor::SyntheticActor;
use crate::actors::unbound_mailbox::UnboundMailbox;
use crate::actors::actor_ref_factory::{ActorRefFactory, ActorSelectError};
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::local_actor_ref::LocalActorRef;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::watcher::WatchingEvents;
use crate::actors::watcher::Watcher;
use crate::actors::scheduler::Scheduler;
use crate::actors::message::Message;
use crate::actors::wrapped_dispatcher::WrappedDispatcher;
use crate::actors::supervision::SupervisionStrategy;
use crate::executors::executor::Executor;
use crate::futures::future::{Future, WrappedFuture};
use std::collections::hash_map::HashMap;
use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex};



pub struct LocalActorSystem {
    /// Actors ids counter. His is used for set up actor name, if it was not be explicitly specified.
    nids: TSafe<usize>,

    /// Default dispatcher. Used if other dispatcher does not explicitly specified.
    dispatchers: TSafe<HashMap<String, TSafe<Dispatcher + Send>>>,

    /// Dead letter actor reference. Sending message through this reference has is very low cost,
    /// because message after drop to the mailbox, is simply destroyed without hes subsequent
    /// execution planning. */
    dead_letters: Option<ActorRef>,

    /// Internal tasks scheduler
    scheduler: TSafe<Scheduler>,

    /// Watcher event bus
    watcher: TSafe<Watcher>,

    /// Root guardian synthetic actor
    root: Option<TSafe<ActorCell>>,

    /// Path of the root guardian actor
    root_path: TSafe<ActorPath>
}

impl LocalActorSystem {
    /// Create new actor system protected by TSafe guard.
    pub fn new() -> LocalActorSystem {
        let cpu_count = num_cpus::get();
        let def_dispatch = tsafe!(DefaultDispatcher::new(cpu_count as u32));
        let mut dispatchers: HashMap<String, TSafe<Dispatcher + Send>> = HashMap::new();
        dispatchers.insert(String::from("default"), def_dispatch.clone());

        let root_path = tsafe!(ActorPath::new("root", None));

        let mut system = LocalActorSystem {
            nids: tsafe!(0),
            dispatchers: tsafe!(dispatchers),
            dead_letters: None,
            root: None,
            root_path: root_path.clone(),
            scheduler: tsafe!(Scheduler::new()),
            watcher: tsafe!(Watcher::new())
        };

        let system_safe = tsafe!(system.clone());


        let root = ActorCell::new(
            system_safe.clone(),
            root_path.clone(),
            tsafe!(SyntheticActor {}),
            0,
            def_dispatch.clone(),
            tsafe!(UnboundMailbox::new()),
            None,
            SupervisionStrategy::Resume
        );
        let root_safe = tsafe!(root);

        let dlp = tsafe!(ActorPath::new("deadLetters", Some(root_path.clone())));
        let dlm = DeadLetters::new();
        let mut dlc = ActorCell::new(
            system_safe.clone(),
            dlp.clone(),
            tsafe!(SyntheticActor {}),
            0,
            def_dispatch.clone(),
            tsafe!(dlm),
            Some(root_safe.clone()),
            SupervisionStrategy::Resume);
        dlc.stopped = false;

        let boxed_dlc = tsafe!(dlc);


        //system.lock().unwrap().dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp)));
        system_safe.lock().unwrap().dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp.clone())));
        system.dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp.clone())));
        system_safe.lock().unwrap().root = Some(root_safe.clone());
        system.root = Some(root_safe);
        boxed_dlc.lock().unwrap().start(boxed_dlc.clone());

        system
    }
}

//TODO у всех ActorSystem убрать метод run

impl ActorRefFactory for LocalActorSystem {



    /// Creates the new actor from specified Props object and with specified name. If name does not
    /// explicitly specified, it will be generate automatically.
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    ///
    fn actor_of(self: &mut Self, props: Props, name: Option<&str>) -> ActorRef {
        let mailbox = tsafe!(UnboundMailbox::new());

        let mut aname: String;

        if name.is_some() {
            aname = name.unwrap().to_string();
        } else {
            aname = self.get_nid();
        }

        let path = tsafe!(ActorPath::new(&aname, Some(self.root_path.clone())));

        let dispatcher: TSafe<Dispatcher + Send> = {
            match &(props.dispatcher)[..] {
                "default" => {
                    let d = self.dispatchers.lock().unwrap();
                    let d = d.get(&String::from("default"));
                    let d = d.as_ref().unwrap();
                    (*d).clone()
                },
                "pinned" => tsafe!(PinnedDispatcher::new()),
                other => {
                    let dispatchers = self.dispatchers.lock().unwrap();
                    let dispatcher = dispatchers.get(&String::from(other));

                    match dispatcher {
                        Some(d) => d.clone(),
                        None => panic!("Dispatcher with name '{}' does not registered", other)
                    }
                }
            }
        };


        let cell = ActorCell::new(
            tsafe!(self.clone()),
            path.clone(),
            props.actor,
            0,
            dispatcher,
            mailbox,
            self.root.clone(),
            props.supervision_strategy
        );
        let boxed_cell = tsafe!(cell);

        {
            let mut root = self.root.as_ref().unwrap().lock().unwrap();
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
        let path = String::from(path);
        let mut segs: VecDeque<&str> = path.split("/").collect();

        let mut selection = Vec::new();
        let mut cur_cell = Some(self.root.as_ref().unwrap().clone());

        if segs.len() == 0 {
            return selection;
        }

        segs.pop_front().unwrap();
        segs.pop_front().unwrap();

        while segs.len() > 0 {
            let seg = segs.pop_front().unwrap();

            let cell = cur_cell.as_ref().unwrap().clone();
            let cell = cell.lock().unwrap();
            let t_cell = cell.childs.get(seg);

            if t_cell.is_some() {
                cur_cell = Some(t_cell.unwrap().clone());
            } else {
                cur_cell = None;
                break;
            }
        }

        if cur_cell.is_some() {
            let cell = cur_cell.unwrap();
            let path = cell.lock().unwrap().path.clone();
            let aref = LocalActorRef::new(cell, path);
            selection.push(Box::new(aref));
        }

        selection
    }

    /// Stop specified actor by it's reference. Suspends actor, cancels all timers, cleans mailbox
    /// and sends to it the PoisonPill message, which will be processed right away after the current
    /// message (if this call will made from actor's message handler) or depending on the stopped
    /// actor state (if it idle it will be stopped immediately, if not, after processing his current
    /// message )
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn stop(self: &mut Self, aref: &mut ActorRef) {
        // Attention, identical code exists in the PoisonPill handler
        let aref_cpy0 = aref.clone();
        let aref_cpy1 = aref.clone();
        let x = aref.cell();
        let mut cell = x.lock().unwrap();
        cell.suspend();
        cell.mailbox.lock().unwrap().clean_up(aref_cpy0, self.dead_letters());
        cell.force_send(aref.cell().clone(), msg!(PoisonPill {}), None, aref_cpy1);
    }

    /// Return deadLetter actor reference
    fn dead_letters(self: &mut Self) -> ActorRef {
        match &self.dead_letters {
            Some(d) =>  {
                (*d).clone()
            }
            _ => panic!("Dead letter is empty")
        }
    }

    /// Register watcher for receive 'watching events' from observed actor
    fn watch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        self.watcher.lock().unwrap().watch(watcher, observed);
    }

    /// Unregister watcher from receive 'watching events' from observed actor
    fn unwatch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        self.watcher.lock().unwrap().unwatch(watcher, observed);
    }
}

impl AbstractActorSystem for LocalActorSystem {

    /// Returns actor system scheduler
    fn get_scheduler(&self) -> TSafe<Scheduler> {
        self.scheduler.clone()
    }

    /// Register new watching event from the specified actor
    fn register_watch_event(&self, from: &ActorRef, event: WatchingEvents) {
        self.watcher.lock().unwrap().register_event(&from, event);
    }

    /// Stops the actor system
    fn terminate(&mut self) {
        let mut root = self.root.as_ref().unwrap().clone();
        root.lock().unwrap().stop(root.clone());
        let d_list = self.dispatchers.lock().unwrap();
        for (_, d) in d_list.iter() {
            d.lock().unwrap().stop();
        }
        self.dead_letters = None;
    }

    /// Adds new dispatcher to the system. For now supporter only default dispatcher replacing
    fn add_dispatcher(&mut self, name: &str, dispatcher: TSafe<Dispatcher + Send>) {
        match name {
            "default" => {
                self.dispatchers.lock().unwrap().get("default").unwrap().lock().unwrap().stop();

                let root = ActorCell::new(
                    tsafe!(self.clone()),
                    self.root_path.clone(),
                    tsafe!(SyntheticActor {}),
                    0,
                    dispatcher.clone(),
                    tsafe!(UnboundMailbox::new()),
                    None,
                    SupervisionStrategy::Resume);
                let root_safe = tsafe!(root);

                let dlp = tsafe!(ActorPath::new("deadLetters", Some(self.root_path.clone())));
                let dlm = DeadLetters::new();
                let mut dlc = ActorCell::new(
                    tsafe!(self.clone()),
                    dlp.clone(),
                    tsafe!(SyntheticActor {}),
                    0,
                    dispatcher.clone(),
                    tsafe!(dlm),
                    Some(root_safe),
                    SupervisionStrategy::Resume);
                dlc.stopped = false;

                let boxed_dlc = tsafe!(dlc);


                self.dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp)));
                self.dispatchers.lock().unwrap().insert(String::from(name), dispatcher);
            },
            _ => {
                let name_str = String::from(name);
                let mut dispatchers = self.dispatchers.lock().unwrap();

                if !dispatchers.contains_key(&name_str) {
                    dispatchers.insert(name_str, dispatcher);
                } else {
                    panic!("Try to add dispatcher with existed name '{}'", name_str)
                }
            }
        }
    }

    /// Returns dispatcher by name
    fn get_dispatcher(&self, name: &str) -> TSafe<Dispatcher + Send> {
        let d = self.dispatchers.lock().unwrap();
        let d = d.get(&String::from(name));
        if d.is_some() {
            (*d.as_ref().unwrap()).clone()
        } else {
            panic!("Attempt to get not registered dispatcher '{}'", name)
        }
    }

    fn get_dispatchers(&self) -> TSafe<HashMap<String, TSafe<Dispatcher + Send>>> {
        self.dispatchers.clone()
    }

    /// Returns dispatcher by name as executor
    fn get_executor(&self, name: &str) -> TSafe<Executor + Send> {
        tsafe!(WrappedDispatcher::new(self.get_dispatcher(name)))
    }

    fn get_nid(&mut self) -> String {
        let mut nids = self.nids.lock().unwrap();
        let name = nids.to_string();
        *nids = *nids + 1;

        name
    }
}

impl Clone for LocalActorSystem {
    fn clone(&self) -> Self {

        let dead_letter: Option<ActorRef> = match &self.dead_letters {
            Some(v) =>
                Some((*v).clone()),
            None =>
                None
        };

        let root: Option<TSafe<ActorCell>> = match &self.root {
            Some(v) =>
                Some((*v).clone()),
            None =>
                None
        };

        LocalActorSystem {
            nids: self.nids.clone(),
            dispatchers: self.dispatchers.clone(),
            dead_letters: dead_letter, //self.dead_letters.clone()
            root,
            root_path: self.root_path.clone(),
            scheduler: self.scheduler.clone(),
            watcher: self.watcher.clone()
        }
    }
}

