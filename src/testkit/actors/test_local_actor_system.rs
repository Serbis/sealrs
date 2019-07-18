//! Test variation of the local actor system.
//!
//! This object is fully mirror of the original actor
//! system version, but additionally extends she with various methods for tests performing.
//! The source code contains special comments which marks code which was fully cloned from the
//! original actor system. All codes outside of this blocks, is code of a tests extensions.

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
use crate::testkit::actors::test_local_actor_ref::TestLocalActorRef;
use crate::testkit::actors::test_probe::TestProbe;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::watcher::WatchingEvents;
use crate::actors::watcher::Watcher;
use crate::actors::message::Message;
use crate::actors::wrapped_dispatcher::WrappedDispatcher;
use crate::executors::executor::Executor;
use crate::actors::scheduler::Scheduler;
use crate::actors::supervision::SupervisionStrategy;
use crate::futures::future::{Future, WrappedFuture};
use std::sync::{Arc, Mutex};
use std::collections::hash_map::HashMap;
use std::collections::vec_deque::VecDeque;


pub struct TestLocalActorSystem {

    // ------- mirror ---------
    nids: TSafe<usize>,
    dispatchers: TSafe<HashMap<String, TSafe<Dispatcher + Send>>>,
    dead_letters: Option<ActorRef>,
    scheduler: TSafe<Scheduler>,
    watcher: TSafe<Watcher>,
    root: Option<TSafe<ActorCell>>,
    root_path: TSafe<ActorPath>,
    // --------- end ----------

    sub: TSafe<Option<ActorRef>>

}

impl TestLocalActorSystem {

    /// Identical to original in all expect than it will automatically starts system. No need call
    /// run manually
    pub fn new() -> TestLocalActorSystem {

        // ------- mirror ---------
        let cpu_count = num_cpus::get() * 2;
        let def_dispatch = tsafe!(DefaultDispatcher::new(cpu_count as u32));
        let mut dispatchers: HashMap<String, TSafe<Dispatcher + Send>> = HashMap::new();
        dispatchers.insert(String::from("default"), def_dispatch.clone());

        let root_path = tsafe!(ActorPath::new("root", None));


        let mut system = TestLocalActorSystem {
            nids: tsafe!(0),
            dispatchers: tsafe!(dispatchers),
            dead_letters: None,
            root: None,
            root_path: root_path.clone(),
            scheduler: tsafe!(Scheduler::new()),
            watcher: tsafe!(Watcher::new()),
            sub: tsafe!(None)
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
            SupervisionStrategy::Resume);
        let root_safe = tsafe!(root);

        let dlp = tsafe!(ActorPath::new("deadLetters", Some(root_path.clone())));
        let dlm = DeadLetters::new();
        let dlc = ActorCell::new(
            system_safe.clone(),
            dlp.clone(),
            tsafe!(SyntheticActor {}),
            0,
            def_dispatch.clone(),
            tsafe!(dlm),
            Some(root_safe.clone()),
            SupervisionStrategy::Resume);

        let boxed_dlc = tsafe!(dlc);


        //system.lock().unwrap().dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp)));
        system_safe.lock().unwrap().dead_letters = Some(Box::new(TestLocalActorRef::new(boxed_dlc.clone(), dlp.clone())));
        system.dead_letters = Some(Box::new(TestLocalActorRef::new(boxed_dlc.clone(), dlp.clone())));
        system_safe.lock().unwrap().root = Some(root_safe.clone());
        system.root = Some(root_safe);
        boxed_dlc.lock().unwrap().start(boxed_dlc.clone());

        system
        // --------- end ----------
    }

    /// Create new TestProbe with specified name
    pub fn create_probe(self: &Self, name: Option<&str>) -> TestProbe {
        TestProbe::new(tsafe!(self.clone()), name)
    }

    /// At next call of actor_of specified ActorRef will be returned instead of really created
    /// actor from a props.
    pub fn replace_actor_of(&mut self, aref: ActorRef) {
        *self.sub.lock().unwrap() = Some(aref);
    }
}

impl ActorRefFactory for TestLocalActorSystem {

    /// Identical to original
    fn actor_of(self: &mut Self, props: Props, name: Option<&str>) -> ActorRef {
        {
            let mut sub = self.sub.lock().unwrap();
            if sub.is_some() {
                let aref = {
                    let r = sub.as_ref().unwrap();
                    (*r).clone()
                };

                *sub = None;
                return aref
            }
        }


        // ------- mirror ---------
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

        Box::new(TestLocalActorRef::new(boxed_cell, path))
        // --------- end ----------

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
            let aref = TestLocalActorRef::new(cell, path);
            selection.push(Box::new(aref));
        }

        selection
    }

    /// Identical to original
    fn stop(self: &mut Self, aref: &mut ActorRef) {

        // ------- mirror ---------
        // Attention, identical code exists in the PoisonPill handler
        let aref_cpy0 = aref.clone();
        let aref_cpy1 = aref.clone();
        let x = aref.cell();
        let mut cell = x.lock().unwrap();
        cell.suspend();
        // +++ cell.actor.timers().cancelAll();
        cell.mailbox.lock().unwrap().clean_up(aref_cpy0, self.dead_letters());
        cell.force_send(aref.cell().clone(), msg!(PoisonPill {}), None, aref_cpy1);
        // --------- end ----------

    }

    /// Identical to original
    fn dead_letters(self: &mut Self) -> ActorRef {

        // ------- mirror ---------
        match &self.dead_letters {
            Some(d) =>  {
                (*d).clone()
            }
            _ => panic!("Dead letter is empty")
        }
        // --------- end ----------
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

impl AbstractActorSystem for TestLocalActorSystem {

    /// Identical to original
    fn get_scheduler(&self) -> TSafe<Scheduler> {
        self.scheduler.clone()
    }

    /// Register new watching event from the specified actor
    fn register_watch_event(&self, from: &ActorRef, event: WatchingEvents) {
        self.watcher.lock().unwrap().register_event(&from, event);
    }

    /// Stops the actor system
    fn terminate(&mut self) {
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
                let dlc = ActorCell::new(
                    tsafe!(self.clone()),
                    dlp.clone(),
                    tsafe!(SyntheticActor {}),
                    0,
                    dispatcher.clone(),
                    tsafe!(dlm),
                    Some(root_safe),
                    SupervisionStrategy::Resume);

                let boxed_dlc = tsafe!(dlc);


                self.dead_letters = Some(Box::new(TestLocalActorRef::new(boxed_dlc.clone(), dlp)));
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

impl Clone for TestLocalActorSystem {
    fn clone(&self) -> Self {

        let dead_letter: Option<ActorRef> = match &self.dead_letters {
            Some(v) => Some((*v).clone()),
            None => None
        };

        let root:  Option<TSafe<ActorCell>> = match &self.root {
            Some(v) =>
                Some((*v).clone()),
            None =>
                None
        };


        TestLocalActorSystem {
            nids: self.nids.clone(),
            dispatchers: self.dispatchers.clone(),
            dead_letters: dead_letter, //self.dead_letters.clone()
            root,
            root_path: self.root_path.clone(),
            scheduler: self.scheduler.clone(),
            watcher: self.watcher.clone(),
            sub: self.sub.clone()
        }
    }
}