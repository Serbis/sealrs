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
use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::testkit::actors::test_local_actor_ref::TestLocalActorRef;
use crate::testkit::actors::test_probe::TestProbe;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::watcher::WatchingEvents;
use crate::actors::watcher::Watcher;
use crate::actors::message::Message;
use crate::actors::wrapped_dispatcher::WrappedDispatcher;
use crate::executors::executor::Executor;
use std::sync::{Arc, Mutex};
use std::collections::hash_map::HashMap;
use crate::actors::scheduler::Scheduler;


pub struct TestLocalActorSystem {

    // ------- mirror ---------
    nids: usize,
    dispatchers: TSafe<HashMap<String, TSafe<Dispatcher + Send>>>,
    dead_letters: Option<ActorRef>,
    scheduler: TSafe<Scheduler>,
    watcher: TSafe<Watcher>,
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
        let mut system = TestLocalActorSystem {
            nids: 0,
            dispatchers: tsafe!(dispatchers),
            dead_letters: None,
            scheduler: tsafe!(Scheduler::new()),
            watcher: tsafe!(Watcher::new()),
            sub: tsafe!(None)
        };

        let system_safe = tsafe!(system.clone());

        let dlp = tsafe!(ActorPath::new("deadLetters"));
        let dlm = DeadLetters::new();
        let dlc = ActorCell::new(system_safe.clone(), dlp.clone(),tsafe!(SyntheticActor {}), 0, def_dispatch.clone(), tsafe!(dlm));

        let boxed_dlc = tsafe!(dlc);


        system_safe.lock().unwrap().dead_letters = Some(Box::new(TestLocalActorRef::new(boxed_dlc.clone(), dlp.clone())));
        system.dead_letters = Some(Box::new(TestLocalActorRef::new(boxed_dlc.clone(), dlp.clone())));
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
            aname = self.nids.to_string();
            self.nids = self.nids + 1;
        }

        let path = tsafe!(ActorPath::new(&aname));

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
            mailbox
        );
        let boxed_cell = tsafe!(cell);

        boxed_cell.lock().unwrap().start(boxed_cell.clone());

        Box::new(TestLocalActorRef::new(boxed_cell, path))
        // --------- end ----------

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

                let dlp = tsafe!(ActorPath::new("deadLetters"));
                let dlm = DeadLetters::new();
                let dlc = ActorCell::new(tsafe!(self.clone()), dlp.clone(),tsafe!(SyntheticActor {}), 0, dispatcher.clone(), tsafe!(dlm));

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

    /// Returns dispatcher by name as executor
    fn get_executor(&self, name: &str) -> TSafe<Executor + Send> {
        tsafe!(WrappedDispatcher::new(self.get_dispatcher(name)))
    }
}

impl Clone for TestLocalActorSystem {
    fn clone(&self) -> Self {

        let dead_letter: Option<ActorRef> = match &self.dead_letters {
            Some(v) => Some((*v).clone()),
            None => None
        };


        TestLocalActorSystem {
            nids: self.nids,
            dispatchers: self.dispatchers.clone(),
            dead_letters: dead_letter, //self.dead_letters.clone()
            scheduler: self.scheduler.clone(),
            watcher: self.watcher.clone(),
            sub: self.sub.clone()
        }
    }
}