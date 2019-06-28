//! Probe of the actor testing system
//!
//! This object is used for controlled interaction with testing actor through sends messages to they
//! and receive messages with some expectations. For more complicated comments, see module level
//! documentation.

use crate::common::tsafe::TSafe;
use crate::testkit::actors::test_local_actor_system::TestLocalActorSystem;
use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::actor::{Actor, HandleResult};
use crate::actors::props::Props;
use crate::actors::watcher::events::Terminated;
use crate::actors::actor_context::ActorContext;
use crate::actors::message::Message;
use std::sync::{Arc, Mutex, Condvar};
use std::any::Any;
use std::time::{ Duration, SystemTime };
use std::thread;

type Matcher = Box<Fn(Message) -> bool + Send>;

pub struct TestProbe {
    /// Probe name
    name: String,

    /// Work actor system
    system: TSafe<TestLocalActorSystem>,

    /// Default timeout for operations
    timeout: Duration,

    /// Internal actor
    inner_actor: Box<ActorRef>,

    /// Probe conditional variable. This var may unlock probe after receive some message or
    /// timer thread
    probe_cvar: Arc<Condvar>,

    /// Mutex for probe_cvar
    probe_cvar_m: Arc<Mutex<bool>>,

    /// Marker for the internal actor which indicates that he may consume the message
    actor_may_work: TSafe<bool>,

    /// Internal action  conditional variable. This var may unlock actor for consume some message
    actor_cvar: Arc<Condvar>,

    /// Internal probe timer
    timer: timer::Timer,

    /// List of current matchers
    matchers: TSafe<Vec<Matcher>>,

    /// Results of apply matchers list to the incoming messages
    match_results: TSafe<Vec<Option<bool>>>,

    /// Sender of the last received message
    last_sender: TSafe<ActorRef>,

    /// Last handled messages
    messages: TSafe<Vec<Message>>
}

impl TestProbe {

    /// Initialize new probe. This is internal construct, do not try use it directly
    pub fn new(system: TSafe<TestLocalActorSystem>, name: Option<&str>) -> TestProbe {
        let probe_cvar = Arc::new(Condvar::new());
        let actor_cvar = Arc::new(Condvar::new());
        let actor_cvar_m = Arc::new(Mutex::new(false));
        let matchers = tsafe!(Vec::new());
        let match_results = tsafe!(Vec::new());
        let actor_may_work = tsafe!(false);
        let last_sender = tsafe!(system.lock().unwrap().dead_letters());
        let messages = tsafe!(Vec::new());

        let actor = TestProbeActor::new(
            probe_cvar.clone(),
            matchers.clone(),
            match_results.clone(),
            actor_cvar.clone(),
             actor_cvar_m,
             actor_may_work.clone(),
            last_sender.clone(),
            messages.clone()
        );
        let actor = tsafe!(actor);
        let inner_actor = system.lock().unwrap().actor_of(Props::new(actor), name);
        let name = if name.is_some() {
            String::from(name.unwrap())
        } else {
            String::from("no_name")
        };

        TestProbe {
            name,
            system,
            timeout: Duration::from_secs(3),
            inner_actor: Box::new(inner_actor),
            probe_cvar,
            probe_cvar_m: Arc::new(Mutex::new(false)),
            actor_may_work,
            actor_cvar,
            timer: timer::Timer::new(),
            matchers,
            match_results,
            last_sender,
            messages
        }
    }

    /// Return internal actor reference
    pub fn aref(&mut self) -> ActorRef {
        self.inner_actor.clone()
    }

    /// Set default expects timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Send message to some actor
    pub fn send(&mut self, target: &mut ActorRef, msg: Message) {
        target.tell(msg, Some(&self.inner_actor))
    }

    /// Reply to the last sender with specified message
    pub fn reply(&mut self, msg: Message) {
        let mut last_sender = self.last_sender.lock().unwrap();
        last_sender.tell(msg, Some(&self.inner_actor))
    }

    /// Expect a single message from an actor. Blocks called thread while message will be received or
    /// timeout was reached. Returns intercepted message instance.
    ///
    /// # Example
    ///
    /// ```
    /// probe.expect_msg(type_matcher!(some_actor::SomeMsg));
    /// let msg = probe.expect_msg(type_matcher!(some_actor::SomeMsgOther));
    /// ```
    ///
    pub fn expect_msg(&mut self, matcher: Matcher) -> Message {
        // Clean early handled messages
        {
            let mut messages = self.messages.lock().unwrap();
            messages.clear();
        }

        // Set current matcher
        *self.matchers.lock().unwrap() = vec![matcher];
        *self.match_results.lock().unwrap() = vec![None];

        // Start timer
        let _guard = self.run_probe_timer(self.timeout);

        // This sleep is need for prevent skip unlocking from the probe actor if it receive message
        // early than this code is executed
        thread::sleep(Duration::from_millis(50));

        // Permits actor process messages
        *self.actor_may_work.lock().unwrap() = true;

        // Notify probe actor for unlock (if it was locked)
        self.actor_cvar.notify_one();

        // Lock current thread for waiting result of timeout
        self.lock();


        let result = self.match_results.lock().unwrap();

        if result[0].is_some() {
            let r = result[0].unwrap();
            if r == false {
                panic!("Test probe '{}' failed in 'expect_msg' with check error ( unexpected message received )", &self.name);
            }
            let mut messages = self.messages.lock().unwrap();
            messages.pop().unwrap()
        } else {
            panic!("Test probe '{}' failed in 'expect_msg' with timeout {} ms", &self.name, self.timeout.as_millis());
        }
    }

    /// Expect an any message in the specified set from an actor. Blocks called thread while message
    /// will be received or timeout was reached. Returns intercepted messages list.
    ///
    /// # Example
    ///
    /// ```
    /// probe.expect_msg_any_of(
    ///     vec![
    ///         type_matcher!(some_actor::SomeMsg0)),
    ///         type_matcher!(some_actor::SomeMsg1)),
    ///         type_matcher!(some_actor::SomeMsg2))
    ///     ]
    /// );
    /// ```
    ///
    pub fn expect_msg_any_of(&mut self, matchers: Vec<Matcher>) -> Message {
        // Clean early handled messages
        {
            let mut messages = self.messages.lock().unwrap();
            messages.clear();
        }

        // Set current matcher
        let mut filled_results = Vec::new();
        for _ in matchers.iter() {
            filled_results.push(None);
        }
        *self.matchers.lock().unwrap() = matchers;
        *self.match_results.lock().unwrap() = filled_results;

        // Start timer
        let _guard = self.run_probe_timer(self.timeout);

        // This sleep is need for prevent skip unlocking from the probe actor if it receive message
        // early than this code is executed
        thread::sleep(Duration::from_millis(50));

        // Permits actor process messages
        *self.actor_may_work.lock().unwrap() = true;



        // Notify probe actor for unlock (if it was locked)
        self.actor_cvar.notify_one();

        // Lock current thread for waiting result of timeout
        self.lock();


        let result = self.match_results.lock().unwrap();
        let mut timeout = false;

        for r in result.iter() {
            if r.is_some() {
                timeout = false;
            }
        }

        if !timeout {
            let mut found = false;
            for r in result.iter() {
                if r.is_some() {
                    if r.unwrap() == true {
                        found = true;
                    }
                }
            }

            if !found {
                panic!("Test probe '{}' failed in 'expect_msg_any_of' with check error ( unexpected message received )", &self.name);
            }

            let mut messages = self.messages.lock().unwrap();
            messages.pop().unwrap()
        } else {
            panic!("Test probe '{}' failed in 'expect_msg_any_of' with timeout {} ms", &self.name, self.timeout.as_millis());
        }
    }

    /// Expect all messages in specified set from an actor. Order of messages is not not significant.
    /// Target message may be altered with some other messages. Test will passed, when all messages
    /// from the list, will be intercepted from input messages stream. Blocks called thread while
    /// message will be received or timeout was reached. Returns intercepted messages list.
    ///
    /// # Example
    ///
    /// ```
    /// probe.expect_msg_all_of(
    ///     vec![
    ///         type_matcher!(some_actor::SomeMsg0)),
    ///         type_matcher!(some_actor::SomeMsg1)),
    ///         type_matcher!(some_actor::SomeMsg2))
    ///     ]
    /// );
    /// ```
    ///
    pub fn expect_msg_all_of(&mut self, matchers: Vec<Matcher>) -> Vec<Message> {
        // Clean early handled messages
        {
            let mut messages = self.messages.lock().unwrap();
            messages.clear();
        }

        let m_len = matchers.len();
        //Internal match results
        let mut internal_results: Vec<Option<bool>> = Vec::new();

        for _ in 0..m_len {
            internal_results.push(None);
        }
        *self.matchers.lock().unwrap() = matchers;


        let started = SystemTime::now();

        // Start timer
        let _guard = self.run_probe_timer(self.timeout);

        // This sleep is need for prevent skip unlocking from the probe actor if it receive message
        // early than this code is executed
        thread::sleep(Duration::from_millis(50));

        // Collect messages
        while true {
            // Set current matcher
            let mut filled_results = Vec::new();
            for _ in  0..m_len {
                filled_results.push(None);
            }
            *self.match_results.lock().unwrap() = filled_results;

            // Permits actor process messages
            *self.actor_may_work.lock().unwrap() = true;

            // Notify probe actor for unlock (if it was locked)
            self.actor_cvar.notify_one();

            // Lock current thread for waiting result of timeout
            self.lock();


            let result = self.match_results.lock().unwrap();
            let elapsed = started.elapsed().unwrap().as_millis();
            let timeout = elapsed >= self.timeout.as_millis();

            if !timeout {
                let mut counter = 0;
                for i in 0..m_len {

                    if internal_results[i].is_none() {
                        let r = result[i].unwrap();
                        if r {
                            internal_results[i] = result[i];
                        }
                    }

                    counter = counter + 1;
                }

                let mut must_cont = false;
                for r in internal_results.iter() {
                    if r.is_none() {
                        must_cont = true;
                    }
                }
                if must_cont {
                    continue;
                }

                //let mut all_ok = false;

                for r in internal_results.iter() {
                    if r.unwrap() == false {
                        panic!("Test probe '{}' failed in 'expect_msg_all_of' with check error ( not all received messages match the patterns )", &self.name);
                    }
                }

                break;

            } else {
                panic!("Test probe '{}' failed in 'expect_msg_all_of' with timeout {} ms", &self.name, self.timeout.as_millis());
            }
        }

        let messages = self.messages.lock().unwrap();
        messages.iter().map(|v| v.clone()).collect()
    }

    /// Expect than no one actor do not send message to this probe in specified time duration
    ///
    /// # Example
    ///
    /// ```
    /// probe.expect_no_msg(Duration::from_secs(1));
    /// ```
    ///
    pub fn expect_no_msg(&mut self, duration: Duration) {
        // Clean last match result
        //*self.match_result.lock().unwrap() = None;

        // Set 'all' matcher
        *self.matchers.lock().unwrap() = vec![matcher! { _v => true }];
        *self.match_results.lock().unwrap() = vec![None];

        // Start timer
        let _guard = self.run_probe_timer(duration);

        // This sleep is need for prevent skip unlocking from the probe actor if it receive message
        // early than this code is executed
        thread::sleep(Duration::from_millis(50));

        // Permits actor process messages
        *self.actor_may_work.lock().unwrap() = true;

        // Notify probe actor for unlock (if it was locked)
        self.actor_cvar.notify_one();

        // Lock current thread for waiting result of timeout
        self.lock();

        let result = self.match_results.lock().unwrap();

        if result[0].is_some() {
            if result[0].unwrap() == true {
                panic!("Test probe '{}' failed in 'expect_no_msg' with check error ( message was received but should not )", &self.name);
            }
        }
    }

    /// Expect termination of the target. Probe must be watch it before cull this expector.
    ///
    /// # Example
    ///
    /// ```
    ///  probe.watch(&target);
    ///  // Some other code
    ///  probe.expect_terminated(&target);
    /// ```
    ///
    pub fn expect_terminated(&mut self, target: &ActorRef) {
        let terminated_matcher = type_matcher!(Terminated);
        // Set current matcher
        *self.matchers.lock().unwrap() = vec![terminated_matcher];
        *self.match_results.lock().unwrap() = vec![None];

        // Start timer
        let _guard = self.run_probe_timer(self.timeout);

        // This sleep is need for prevent skip unlocking from the probe actor if it receive message
        // early than this code is executed
        thread::sleep(Duration::from_millis(50));

        // Permits actor process messages
        *self.actor_may_work.lock().unwrap() = true;

        // Notify probe actor for unlock (if it was locked)
        self.actor_cvar.notify_one();

        // Lock current thread for waiting result of timeout
        self.lock();


        let result = self.match_results.lock().unwrap();

        if result[0].is_some() {
            let r = result[0].unwrap();
            if r == false {
                panic!("Test probe '{}' failed in 'expect_terminated' with check error ( target must be terminated but instead sent some message )", &self.name);
            } else {
                if self.last_sender.lock().unwrap().path() != target.path() {
                    panic!("Test probe '{}' failed in 'expect_terminated' with check error ( actor terminated, but not target )", &self.name);

                }
            }
        } else {
            panic!("Test probe '{}' failed in 'expect_terminated' with timeout {} ms", &self.name, self.timeout.as_millis());
        }
    }

    /// Subscribe probe fot watching events of target
    pub fn watch(&mut self, target: &ActorRef) {
        self.system.lock().unwrap().watch(&self.inner_actor, target);
        //unwatch in drop and actor stop
    }

    /// Internal locker for awaiting messages from the internal actor or timeout
    fn lock(&mut self) {
        self.probe_cvar.wait( self.probe_cvar_m.lock().unwrap());
    }

    /// Run probe timer witch must be unlock probe_cvar, that indicated what expectation does not
    /// satisfied with specified timeout
    fn run_probe_timer(&mut self, timeout: Duration) -> timer::Guard {
        let cvar = self.probe_cvar.clone();
        self.timer.schedule_with_delay(chrono::Duration::from_std(timeout).ok().unwrap(), move || {
            cvar.notify_one();
            //println!("xxx");
        })
    }
}

impl Drop for TestProbe {
    fn drop(&mut self) {
        self.system.lock().unwrap().stop(&mut self.inner_actor.clone());
        *self.actor_may_work.lock().unwrap() = true;
        self.actor_cvar.notify_one();
    }
}

/// Internal test actor
///
/// This actor receive all messages, pass it through list of matchers functions, and create mapped
/// results list of each matcher result. On this results list, probe will be to make decisions
/// about passing concrete test action
///
struct TestProbeActor {
    probe_cvar: Arc<Condvar>,
    matchers: TSafe<Vec<Matcher>>,
    match_results: TSafe<Vec<Option<bool>>>,
    actor_cvar: Arc<Condvar>,
    actor_cvar_m: Arc<Mutex<bool>>,
    actor_may_work: TSafe<bool>,
    last_sender: TSafe<ActorRef>,
    messages: TSafe<Vec<Message>>
}

impl TestProbeActor {
    pub fn new(
        probe_cvar: Arc<Condvar>,
        matchers: TSafe<Vec<Matcher>>,
        match_results: TSafe<Vec<Option<bool>>>,
        actor_cvar: Arc<Condvar>,
        actor_cvar_m: Arc<Mutex<bool>>,
        actor_may_work: TSafe<bool>,
        last_sender: TSafe<ActorRef>,
        messages: TSafe<Vec<Message>>) -> TestProbeActor {

        let _test_matcher = |_v: &Box<Any + Send>| {
            true
        };
        TestProbeActor {
            probe_cvar,
            matchers,
            match_results,
            actor_cvar,
            actor_cvar_m,
            actor_may_work,
            last_sender,
            messages
        }
    }

    fn lock(&mut self) {
        self.actor_cvar.wait( self.actor_cvar_m.lock().unwrap());
    }
}

impl Actor for TestProbeActor {

    fn receive(&mut self, msg: Message, ctx: ActorContext) -> HandleResult {
        if *self.actor_may_work.lock().unwrap() == false {
            self.lock();
        }
        *self.actor_may_work.lock().unwrap() = false;

        //Set the message sender
        *self.last_sender.lock().unwrap() = ctx.sender.clone();

        let matchers = self.matchers.lock().unwrap();
        let mut match_results = self.match_results.lock().unwrap();

        let mut counter = 0;
        for m in matchers.iter() {
            if match_results[counter] == None {
                let result = (m)(msg.clone());
                if result {
                    let mut messages = self.messages.lock().unwrap();
                    messages.push(msg.clone());
                }
                match_results[counter] = Some(result);
            }

            counter = counter + 1;
        }

        self.probe_cvar.notify_one();

        Ok(true)
    }

}