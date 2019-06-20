//! Test framework for actors
//!
//! # Table of Contents
//! 1. [Introduction](#introduction)
//! 2. [TestActorSystem](#testactorsystem)
//! 3. [TestActorRef](#testactorref)
//! 4. [TestProbe](#testprobe)
//! 5. [Examples](#examples)
//!
//! # Introduction
//!
//! Amazing opportunity of the untyped actors consists in the feature, that actor may by tested in
//! fully isolated test environment. All outside environment of the actor will be mocked, but he
//! itself will be think, that it work in the real world system, and never will known about that
//! all they environment is emulated. In order to achieve this effect, testkit use few elements.
//! Let's look on they!
//!
//! # TestLocalActorSystem
//!
//! This is a special test realization of the actor system (actor system itself is abstract interface,
//! and may have any realizations). She fully duplicate functional of the regular actor system, but
//! add to her various functions for needs of testing. She may be created as regular actor system.
//! Only difference consist in that this system, does not need run explicitly. She is run itself
//! right after creation.
//!
//! ```
//! let system = TestLocalActorSystem::new();
//! ```
//!
//! # TestLocalActorRef
//!
//! Special ActorRef which produced by the TestLocalActorSystem. Like the last, he is fully
//! duplicate functions of the regular reference with appending an additional functions.
//!
//! ### Access to the actor's state
//!
//! TestLocalActorRef allow you get direct access to the internal actor object and see it's state.
//! This state is immutable, but may be used for assertions about states change after processing
//! some test actions. Link to the actor is accessed as "actor" attribute of the reference object
//! and represents as Any type object. For use it, you need downcast they to the concrete type.
//! Because it is strongly complex operation, testkit present special macro for this:
//!
//! ```
//! in_state! (target, SomeType, actor => {
//!     assert_eq!(actor.data, 599);
//! });
//! ```
//!
//! It this expression, 'target' is the link to the ActorRef, 'SomeType' is the actor's concrete
//! type and the 'actor' is the casted variable for inner use.
//!
//! Important note! For use this feature, target actor must implement as_any method of the 'Actor'
//! trait. Without satisfaction this requirement, type cast operation over actor object will be
//! completed with panic! This is example of simplest implementation of this method:
//!
//! ```
//!  fn as_any(self: &Self) -> &Any { self }
//! ```
//!
//! ### Substitution of created actors
//!
//! Method replace_actor_of do the next thing. He sets special flag in the actor system, which
//! indicates, that when next actor_of will be called, instead of creating new actor, specified
//! ActorRef will be return. This feature is used when tested actor must create some other
//! actor, and you want to intercept this action and inject a test probe actor instead a real.
//! After that, you will receive all message addressed to that actor and may respond to it.
//! For example, it is possible to present the following situation. You send a message to a
//! target actor. He handle that message, creates new other actor and send to him some other
//! message. You may intercept this interaction in such code:
//!
//! ```
//! system.replace_actor_of(fiction.aref());
//! probe.send(&mut target, msg!(commands::SomeMessages { }));
//! fiction.expect_msg(type_matcher!(other_actor_commands::SomeMessages));
//! ```
//!
//! # TestProbe
//!
//! This is a heart part of actor's testing. TestProbe is an object, which constructed over inner
//! actor and allows for external code, interact with it in a blocking manner. Probe may by created
//! from the test actor system:
//!
//! ```
//! let probe = system.create_probe(Some("my_probe"));
//! ```
//!
//! After that, with him may be performed various operations.
//!
//! ### Send
//!
//! Sends specified message the specified actor on behalf of the probe actor. Target actor may
//! respond on this message, and response will be received by the probe actor.
//!
//! ### Expectations
//!
//! The main feature of the Test Probe consists in than it may expect some messages and fail test
//! if this messages does not satisfy for specified requirements. For this task, probe presents few
//! method which names starts with 'expect' prefix. Each of this methods receive as arg one or more
//! anonymous functions which is called 'matcher'. Matcher receive a message as argument and return
//! a bool value witch determine result of matching test for a message. All expectors functions is
//! blocks the called thread. After block, inner actor
//! must receive a message/messages and pass them through early set matchers. If the inner actor
//! receive a messages, expector function will be unlocked and decide what to do with results of
//! matching  - pass test or fail it. If the inner actor does not receive any messages in the
//! specified timeout, expector function also will be unlocked, and test will be failed by timeout
//! error. Default timeout is set to 3 seconds. This time may be tuned though set_timeout method.
//! Now let's define list of existed expects.
//!
//! * expect_msg - Expects a single message. Returns an instance of an intercepted message.
//! * expect_msg_any_of - Expects any message from the presented set. Returns an instance of an
//! intercepted message.
//! * expect_msg_all_off - Expects all message from the presented set. Order of the messages is not
//! determined. Also target messages may to alternate with message does not from set. Returns a
//! list of instances of an intercepted messages.
//! * expect_no_msg - Expects that no one message will be received in the specified time interval.
//! * expect_terminated - Expects that specified actor will be stopped. Before perform this
//! expectations, probe must be watch target actor:
//!
//! ```
//! probe.watch(&target);
//! ```
//!
//! Now what with matchers. Matcher is a function with next signature:
//!
//! ```text
//! type Matcher = Box<Fn(Message) -> bool + Send>;
//! ```
//!
//! Typical operations, which do a matchers, consists in check of message type through it's
//! downcasting and pass some clarifying checks, such us more precise pattern matching. Example of
//! typical matcher function is presented below.
//!
//!
//! ```
//! Box::new(|v: Message| {
//!     if let Some(m) = v.get().downcast_ref::<logger::Log>() {
//!         if m.text.len() > 100 {
//!             match m.target {
//!                 logger::LogTarget::StdOut => true,
//!                 _ => false
//!             }
//!         } else {
//!             false
//!         }
//!     } else {
//!         false
//!     }
//!});
//! ```
//!
//! Of course this is a very dirty code and testkit presents few macros, in order to tests code looks
//! like more clean. First such macro is 'matcher!'. It is the simple wrapper for hiding header
//! details of the function.
//!
//! ```
//! matcher!(v => {
//!     if let Some(m) = v.get().downcast_ref::<logger::Log>() {
//!         if m.text.len() > 100 {
//!             match m.target {
//!                 logger::LogTarget::StdOut => true,
//!                 _ => false
//!             }
//!         } else {
//!             false
//!         }
//!     } else {
//!         false
//!     }
//! });
//! ```
//!
//! Second type, constructs a function, which checks a message for match to the specified type.
//!
//! ```
//! type_matcher!(logger::Log);
//! ```
//!
//! Third type, constructs a function, which checks a message for match to the specified simple
//! pattern (without guards).
//!
//! ```
//! pat_matcher!(logger::Log => logger::Log { text: _, target: logger::LogTarget::StdOut });
//! ```
//!
//! Fourth type, is most useful. It constructs a function, which checks a message for match to the
//! specified type and after that apply to the casted message some user code with additional checks.
//! This is useful when simple pattern match checks does not sufficient. For example you need call a
//! method of the object which lives as field of the message.
//!
//! ```
//! extended_type_matcher!(logger::Log, v => {
//!     if v.get().text.len() > 100 {
//!         true
//!     } else {
//!         false
//!     }
//! });
//! ```
//!
//! Summarizing the paragraph, below present example of usage expector-functions with described
//! matchers.
//!
//! ```
//! probe.expect_msg(type_matcher!(logger::Log));
//!
//! // or
//!
//! probe.expect_msg_any_of(
//!     vec! [
//!         pat_matcher!(logger::Log => logger::Log { text: _, target: logger::LogTarget::StdOut }),
//!         pat_matcher!(logger::Log => logger::Log { text: _, target: logger::LogTarget::File })
//!     ]
//! );
//!
//! ```
//!
//! ### Reply
//!
//! This function permits to you send a message to the sender of last received message. In other
//! words, you may respond to the sender. This allow to you construct message passing chains with
//! target actor. You send message to the target actor, receive response from it and if it is
//! matched, reply to them with new message, and again wait a message and so on.
//!
//! ```
//! probe.send(target.clone(), msg!(some_actor::DoFirstAction { }));
//! probe.expect_msg(type_matcher!(some_actor::OkForFirstAction));
//! probe.send(target.clone(), msg!(some_actor::DoSecondAction { }));
//! probe.expect_msg(type_matcher!(some_actor::OkForSecondAction));
//!
//! ```
//!
//! ### DI testing
//!
//! Internal actor is a regular actor existed under standard ActorRef. This ActorRef may be
//! extracted, cloned and passed as dependency to the tested actor. This operation is names as
//! dependency injection testing. Injected test actor reference will be receive all message which
//! will be addressed to emulated dependency. For see hiw it work, remember  example with logger
//! from the main doc. Logger actor receive few actors as dependency's, which realize concrete
//! logic of writing logs. Logger after receive the Log message, determines log target and send
//! command for write log, to the specified writer. All this logic may tested by fully transparent
//! way:
//!
//! ```
//! let mut probe = system.create_probe(Some("probe"));
//! let mut stdout_writer = system.create_probe(Some("stdout_writer"));
//! let mut file_writer = system.create_probe(Some("file_writer"));
//! let mut target = system.actor_of(logger::props(file_writer.aref(), stdout_writer.aref()), Some("logger"));
//!
//! probe.send(target.clone(), msg!(logger::Log { text: String::from("_"), target: logger::LogTarget::File }));
//! file_writer.expect_msg(type_matcher!(file_writer::Write));
//!
//! probe.send(target.clone(), msg!(logger::Log { text: String::from("_"), target: logger::LogTarget::StdOut }));
//! stdout_writer.expect_msg(type_matcher!(stdout_writer::Write));
//! ```
//!
//! # Helper macros
//!
//! In the testkit exists few macros which simplify some aspects of testing.
//!
//! * cast! - Casts some Message to the specified type and call a user function with this value. Macro
//! may be used in two modes. First as validator, when you getting access to the internal data of
//! a message. Second as extractor of data from a messages, because he may return values from itself
//! based on the data from a message.
//!
//! ```
//! // As validator
//! cast!(msg, responses::MsgResponse, m => {
//!     assert_eq!(m.data, 99);
//! });
//!
//! // As extractor
//! let data = cast!(msg, responses::MsgResponse, m => {
//!     m.data;
//! });
//! ```
//!
//! # Examples
//!
//! In the 'examples/testkit/bagsman.rs' presented all types of test actions which you may do with
//! this testkit. Initially all tests in this file is passable, but in the commented blocks of code,
//! defined various situations with tests which will be failed for some reasons. Uncomment it fow see
//! what to happen.

#[macro_use]
pub mod macrodef;
pub mod test_local_actor_system;
pub mod test_local_actor_ref;
pub mod test_probe;
pub mod prelude;


