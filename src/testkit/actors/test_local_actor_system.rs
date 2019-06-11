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
use std::sync::{Arc, Mutex};
use crate::actors::scheduler::Scheduler;


//TODO способ проверки создания актора можно реализовать через expect_actor_creation. В нем выставляется специальный флаг с рефом, который будет возвращщен после следующего вызова actor_of
pub struct TestLocalActorSystem {

    // ------- mirror ---------
    nids: usize,
    pub dispatcher: TSafe<DefaultDispatcher>,
    dead_letters: Option<ActorRef>,
    scheduler: TSafe<Scheduler>,
    watcher: TSafe<Watcher>
    // --------- end ----------

}

impl TestLocalActorSystem {

    /// Identical to original in all expect than it will automatically starts system. No need call
    /// run manually
    pub fn new() -> TSafe<TestLocalActorSystem> {

        // ------- mirror ---------
        let cpu_count = num_cpus::get();
        let mut dispatcher = DefaultDispatcher::new(cpu_count as u32);
        //dispatcher.run();
        let dispatcher = tsafe!(dispatcher);
        let mut system = TestLocalActorSystem {
            nids: 0,
            dispatcher: dispatcher.clone(),
            dead_letters: None,
            scheduler: tsafe!(Scheduler::new()),
            watcher: tsafe!(Watcher::new())
        };

        let system = tsafe!(system);

        let dlp = tsafe!(ActorPath::new("deadLetters"));
        let dlm = DeadLetters::new();
        let dlc = ActorCell::new(system.clone(), dlp.clone(),tsafe!(SyntheticActor {}), 0, dispatcher.clone(), tsafe!(dlm));

        let boxed_dlc = tsafe!(dlc);


        system.lock().unwrap().dead_letters = Some(Box::new(TestLocalActorRef::new(boxed_dlc.clone(), dlp)));
        boxed_dlc.lock().unwrap().start(boxed_dlc.clone());

        system
        // --------- end ----------
    }

    /// Create new TestProbe with specified name
    pub fn create_probe(self: &Self, name: Option<&str>) -> TestProbe {
        TestProbe::new(tsafe!(self.clone()), name)
    }
}

impl ActorRefFactory for TestLocalActorSystem {

    /// Identical to original
    fn actor_of(self: &mut Self, props: Props, name: Option<&str>) -> ActorRef {

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

        let mut cell = ActorCell::new(
            tsafe!(self.clone()),
            path.clone(),
            props.actor,
            0,
            self.dispatcher.clone(),
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
        cell.force_send(aref.cell().clone(), Box::new(PoisonPill {}), None, aref_cpy1);
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
    fn watch(&mut self, watcher: &ActorRef, mut observed: &ActorRef) {
        self.watcher.lock().unwrap().watch(watcher, observed);
    }

    /// Unregister watcher from receive 'watching events' from observed actor
    fn unwatch(&mut self, mut watcher: &ActorRef, mut observed: &ActorRef) {
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
}

impl Clone for TestLocalActorSystem {
    fn clone(&self) -> Self {

        let dead_letter: Option<ActorRef> = match &self.dead_letters {
            Some(v) => Some((*v).clone()),
            None => None
        };
        TestLocalActorSystem {
            nids: self.nids,
            dispatcher: self.dispatcher.clone(),
            dead_letters: dead_letter, //self.dead_letters.clone()
            scheduler: self.scheduler.clone(),
            watcher: self.watcher.clone()
        }
    }
}