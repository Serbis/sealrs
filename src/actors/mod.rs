//! Actor based asynchronous runtime
//!
//! # Table of Contents
//! 1. [Introduction](#introduction)
//! 2. [Perspective features](#perspective-features)
//! 3. [Architecture elements](#architecture-elements)
//! 4. [Basic operations](#basic-operations)
//! 5. [Lifetime hooks](#lifetime-hooks)
//! 6. [Stopping actors](#stopping-actors)
//! 7. [Actor communications](#actor-communications)
//! 8. [ActorRef sharing](#actorref-sharing)
//! 9. [More about DeadLetters](#more-about-deadletters)
//! 10. [Timers](#timers)
//! 11. [Watching](#watching)
//! 12. [Ask](#ask)
//!
//! # Introduction
//! This runtime is realize the classical actor paradigm actively used in the actor based languages,
//! such as Erlang and actors framework, such as Akka. To be frank, this module is heavily relies on
//! the last and actively use principles and abstractions submitted in this framework.
//!
//! At this moment, module presents next features:
//! * Untyped actors with exchanged the messages derived from Any trait
//! * Actor lifetime management
//! * DeadLetters
//! * TestKit
//! * Timers
//! * Watching
//!
//! # Perspective features:
//! Under this features already exists architectural basis, and their implementation is a question
//! of my free time.
//!
//! * Mailboxes with user defined functional
//! * Various dispatchers realizations
//! * Supervising
//! * FSM
//! * Distributed actor systems
//!
//!
//! # Architecture elements
//!
//! Before we go to the concrete examples of usage, you mast understand the internal architecture of
//! the actors runtime. Without this, you will do not understand many of errors which may cause at
//! work with the library and it will be seems for you as bags, but it's doesn't.
//!
//! Actor runtime contains three big elements:
//! * Runtime - contains objects used of actor at execution
//! * System -  environment of actors representation . Controls actors lifetime (start, stop, and
//! etc.). Contains default and other system elements.
//! * Actor - objects providing actor execution in the runtime
//!
//! Runtime consists two object:
//! * Mailbox - queue protected from race condition. This is an interface, which presents set of
//! some basic methods for work with the internal queue. In this queue drops all messages passed
//! to the actor before his actual processing. In the library exists few standard realization of
//! this interface:
//!
//!     * UnboundedMailbox - mailbox without any queue size restrictions. This mailbox is used by
//! default at the time of actor creation, if other mailbox type, does not specified explicitly.
//!
//! * Dispatcher - entity which process the messages. Message processing may be planned in it, and
//! after that, dispatcher decides himself, how and when run his processing. Message processing is
//! the operation, when receive method of actor is called with processed message and prepared
//! context. Dispatcher is an interface, and him concrete behavior base on it's implementation. In
//! the library exists few standard realization of this interface:
//!
//!     * DefaultDispatcher - threads pinned dispatcher, in which message of each concrete actor,
//! processed in only one OS thread. This feature allows actors has internal state without any
//! protections from race condition. This dispatcher is used by default at the time of actor
//! creation, if other dispatcher type, does not specified explicitly.
//!
//!  Runtime consists five object:
//! * Actor - user defined structure implemented with the Actor trait. This object contains
//! application logic of program constructed in the actors style. Him has one mandatory function for
//! realization (receive) and few optional (preStart, postStop an others).
//! * ActorRef - representation of the actor link. This is an interface, through which users code
//! may interact with underlayed actor. This interface is an encapsulate of interaction with the
//! underlying ActorCell (described below). ActorRef may be cloned as many time as you want, safe
//! shared between actors and other thread. This feature ensured by the fact, that all instances of
//! the cloned ref, will link to the single protected ActorCell object.
//! * ActorCell - system actor representation. Coordinates all interaction with actor from the
//! outside. This object contains the current lifetime state of an actor. In him stored mailbox and
//! link to the dispatcher, in which will be processed messages from the mailbox. Functional it
//! performs, start, stop and suspend actor. Interacts with dispatcher for planning messages
//! processing and etc. Need make a comment about the mailbox and dispatcher instances. ActorCell
//! has always it's own instance if the mailbox, which will created with the other elements of the
//! actor at the creation stage of him. Unique of the dispatcher depends on it's type. Some
//! dispatchers has a shared nature. They may be user with few actors at the same time. Others is
//! unique for each actor instance. Example of the first type dispatcher is DefaultDispatcher.
//! Second is PinnedDispatcher (in developing stage, used in the strategy - os thread per actor).
//! * ActorPath - representation of the actor path in the actors system. This object is used for
//! comparing actor references, paths printing in the debug messages and other internal needs.
//! * ActorContext - actor environment at processing message. This struct conatins message sender
//! reference, self actor reference, link to the system on witch actor is executed and other useful
//! values and methods.
//!
//! System consists two object:
//! * ActorSystem - object represents the actor system. It used for creating, stopping actors and
//! some other service operations performing.
//! * Dead Letter - special sort of the mailbox in which all enqueued messages is logged and after
//! that destroyed. Never is planned. Used to dispose of undelivered messages for various reasons.
//!
//! # Basic operations
//!
//! Now, after we will learn set of the basis framework elements, let's try to write some practical
//! code. First, we need some implementation of an actor. Let's create actor to which we may send
//! message with some text, and actor will print it. As side-effect, actor will be count of
//! printed characters.
//!
//! ```
//! use crate::actors::prelude::*;
//! use std::any::Any;
//! use std::sync::{Mutex, Arc};
//! use match_downcast::*;
//!
//! pub fn props() -> Props {
//!     Props::new(tsafe!(BasicActor::new()))
//! }
//!
//! pub struct Print {
//!     pub text: String,
//! }
//!
//! pub struct BasicActor {
//!     printed_chars: usize
//! }
//!
//! impl BasicActor {
//!     pub fn new() -> BasicActor {
//!         BasicActor {
//!             printed_chars: 0
//!         }
//!     }
//! }
//!
//! impl Actor for BasicActor {
//!
//!     fn receive(self: &mut Self, msg: Message, _ctx: ActorContext) -> bool {
//!         let msg = msg.get();
//!         match_downcast_ref!(msg, {
//!             m: Print => {
//!                 self.printed_chars = self.printed_chars + m.text.len();
//!                 println!("{}", m.text);
//!             },
//!             _ => return false
//!         });
//!
//!         true
//!     }
//! }
//! ```
//!
//! Next we write the code which use this actor:
//!
//! ```
//! let mut system = LocalActorSystem::new();
//!
//! let mut printer = system.lock().unwrap()
//!     .actor_of(basic_actor::props(), Some("printer"));
//!
//! let msg = msg!(basic_actor::Print { text: String::from("Hello world!") });
//! printer.tell(msg, None);
//! ```
//!
//! At first line, we create new actor system. Internally, this operation creates default dispatcher
//! and DeadLetter synthetic actor. Should pay attention on the following fact. ActorSystem have type
//! TSafe<LocalActorSystem>. TSafe is macro which expands to Arc<Mutex\<T\>>. Last is the standard rust
//! thread safe shared pointer. This fact means for as, that we will mast lock this pointer before
//! use it. Omissions in the logic of the this pointers unlocking, when you use him outside of the
//! actor system, will guaranteed lead to some sort of deadlocks. For this reason, you must pay
//! extreme attention for lock management of this object if you plan to use this pointer in the
//! other places, rather than actors. In normal conditions, his is used only at the actor system
//! deploying stage. All other interactions with system will be performs in the actors, and does not
//! may cause deadlock.
//!
//! At next line, we create the early defined actor. actor_of functions is receive Pops object as
//! first argument, and actor name as second. Actor name is optional, and it may be set to None.
//! In this case, name will be generated automatically. Props object is indicates the system, which
//! actor to create, and with which params. We will come across this structure more than once as
//! the material is presented. For now, we simple creates new actor instance and put it's to the
//! Props object. As a result of the actor_off call, we will receive actor link object - ActorRef.
//! Internally this call, besides the ActorRef, will create next objects: ActorCell, Mailbox and
//! Dispatcher if it was specified and has unique nature. After performs all internal operations,
//! library code will call preStart function, which will be described later in this doc.
//!
//! And at last stage, we may send the Print message to the actor.We create the new Print structure,
//! uses for it the msg! macro and then send her to the actor through tell function. The 'msg!'
//! macros is creates the special wrapper for messages, which may contain any data type and he may
//! be safely shared between threads. Tell function receive message as first
//! argument and sender actor reference as second. Last is optional, because if we send message
//! outside of the actor system we do not someone whom may be represents as sender. If you perform
//! this operation from the actor, you should be set ctx.self_ to this value. But about this we
//! say later. When you call tell function, ActorCell get passed message to the mailbox and
//! indicates dispatcher schedule this message for processing.
//!
//! Ok, message in the mailbox, what happens with him further? Next, the dispatcher, when he
//! will be ready for it, extract message from the mailbox and message processing stage is starts.
//! First, the message is heading to the internal receive function. His task, determine service message
//! (such as PoisonPill, witch stops the actor). If it is, it will be processed, some service
//! actions will be performed and message will dropped. He will does not reach the  receive function
//! of the target actor. If this message is not a service message, actor.receive function will be
//! called. This function must return a boolean value, which indicates fact, was  handled message or
//! not. If the message was handled, message processing logic is end, else the message will be dropped
//! to the DeadLetter.
//!
//! Now consider the the function of messages receive of our actor. It have three args. First arg
//! is the link to self object which state is fully mutable. Second args is the received message.
//! Last arg is the message processing context. Message represents as boxed Any type and may be
//! matched be any comfortable way for you. I use for this the crate 'match-downcast' which allows
//! keep code clear at downcast operations. From the context, for now, only one object is
//! interesting - sender. What does he point to? At DeadLetter, because at tell operation, we was
//! specify None as sender reference.
//!
//! # Lifetime hooks
//! Actor has few lifetime hooks which may be used for various operations:
//! * preStart - called after full actor initialization and before allowing reception of
//! messages. At this stage already exists actor context, which may be used for creating new actors
//! or sending message for himself. Also is available actor's internal state for mutating.
//! * postStop - called after full stop of actor. This means, that all message passed to the actor
//! was processed or dropped and it will not receive any new messages further. This is the last
//! point of interaction with internal actor's state. At this stage actor context also exists and
//! may be user to perform various operations.
//!
//! For example let's add this hooks to our basic actor:
//!
//! ```
//! ...
//!
//! fn pre_start(self: &mut Self, ctx: ActorContext) {
//!     println!("BasicActor is started")
//! }
//!
//! fn post_stop(self: &mut Self, ctx: ActorContext) {
//!     println!("BasicActor is stopped")
//! }
//!
//! ...
//! ```
//!
//! After you run this modified example, before printing the passed message, in the stdout you will
//! see 'BasicActor is started'. postTop  will triggered after actor will be stopped. This operation
//! we will learn below.
//!
//! # Stopping actors
//! Actor can  be stopped in two way:
//! * Send PoisonPill - this is a special service message, located in the actor submodule.
//! If you send this message to an actor, it will be enqueued as an any regular message. But when
//! dispatcher will face with him, he will drop all messages in the mailbox after it, and starts
//! actor stopping procedure.
//!
//!     ```
//!     bench.tell(msg!(actor::PoisonPill { }), None);
//!     ```
//!
//! * ActorSystem::stop(ActorRef) - this call immediately stops the specified actor. He lead to block mailbox
//! of the actor, flush all it's messages and mark him as stopped. If this call happens from a
//! message handler, current message will be processed, but all subsequent does not. Internally this
//! call does the following. He suspends the actor, clean his mailbox and prepend to it ths single
//! PoisonPill message. This will lead to that after processing the current message dispatcher will
//! immediately come across a message PoisonPill and stops the actor. Similar situation will be
//! happened and if this call occurs outside of actor system. In this case, if target actor is in
//! process of some message, it will be processed and after that actor will be stopped.
//!
//!     ```
//!     system.lock().unwrap().stop(bench);
//!     ```
//!
//! # Actor communications
//! For consideration of this topic, we need some more complicated example. In the 'example'
//! submodule located module named logger. This is more complex program then basic example, but this
//! much more real world example of actors usage. Here contains three actors which realize logger,
//! in which we may indicate logging target. Log target may be StdOut or File. Logger actor contains
//! links to the writers, whom directly realize logging logic to him target. Task of the logger
//! actor is  receive Log message, determine target writer actor and send Write message to him.
//! Writer actor receive this message, performs write and respond to the sender with Ok message.
//! Logger actor receive this message, increment internal counter of write chars and print info
//! text about this action to console. Below are given 'receive' functions of this actors.
//! Surrounding code is redundant for this doc, and you may see him in in the source files.
//!
//! ```
//!// Logger
//!fn receive(self: &mut Self, msg: Message, ctx: ActorContext) -> bool {
//!    let msg = msg.get();
//!    match_downcast_ref!(msg, {
//!            m: Log => {
//!                match m.target {
//!                    LogTarget::File => {
//!                        let msg = msg!(file_writer::Write { text: m.text.to_string() });
//!                        self.file_writer.tell(msg , Some(&ctx.self_))
//!                    },
//!                    LogTarget::StdOut => {
//!                        let msg = msg!(stdout_writer::Write { text: m.text.to_string() });
//!                        self.stdout_writer.tell(msg, Some(&ctx.self_))
//!                    }
//!                };
//!            },
//!            m: file_writer::Ok => {
//!                println!("File logger write '{}' chars", m.chars_count);
//!                self.chars_counter = self.chars_counter + m.chars_count;
//!            },
//!            m: stdout_writer::Ok => {
//!                 println!("Stout logger write '{}' chars", m.chars_count);
//!                self.chars_counter = self.chars_counter + m.chars_count;
//!            },
//!            _ => return false
//!        });
//!
//!    true
//!}
//!
//!
//!//StdoutWriter
//!fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> bool {
//!    let msg = msg.get();
//!    match_downcast_ref!(msg, {
//!            m: Write => {
//!               println!("{}", m.text);
//!               let resp = msg!(Ok { chars_count: m.text.len() });
//!               ctx.sender.tell(resp, Some(&ctx.self_));
//!            },
//!            _ => return false
//!        });
//!
//!    true
//!}
//!
//!
//! //FileWriter
//!fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> bool {
//!    let msg = msg.get();
//!    match_downcast_ref!(msg, {
//!            m: Write => {
//!               fs::write(&self.file, m.text.as_bytes());
//!               let resp = msg!(Ok { chars_count: m.text.len() });
//!               ctx.sender.tell(resp, Some(&ctx.self_));
//!            },
//!            _ => return false
//!        });
//!
//!    true
//!}
//! ```
//!
//! And code which use this actors:
//!
//! ```
//! let mut system = LocalActorSystem::new();
//!
//! let mut logger =  {
//!     let mut system = system.lock().unwrap();
//!     let file_writer = system.actor_of(file_writer::props("/tmp/log"), Some("file_writer"));
//!     let stdout_writer = system.actor_of(stdout_writer::props(), Some("stdout_writer"));
//!     system.actor_of(logger::props(file_writer, stdout_writer), Some("logger"))
//! };
//!
//! logger.tell(msg!(logger::Log { text: String::from("To file log"), target: logger::LogTarget::File }), None);
//! logger.tell(msg!(logger::Log { text: String::from("To stdout log"), target: logger::LogTarget::StdOut }), None);
//! ```
//!
//! This code demonstrate two patterns and operation as part of this library:
//! * Dependency Injection - at deploying stage of the system, we create two actors (writers) and
//! inject it to the logger actor through him constructor (which is hidden by the props function).
//! * Context using - demonstrated actor context usage for sending messages to another actor and
//! respond to messages.
//!
//! # ActorRef sharing
//! In the previous example we was moved created references to the logging writers to the
//! constructor of logger actor. But what about if you want to use this references later in your
//! code? For this, ActorRef's may by simply cloned.
//!
//! ```
//!let (mut logger, mut file_writer, mut stdout_writer) = {
//!     let mut system = system.lock().unwrap();
//!     let file_writer = system.actor_of(file_writer::props("/tmp/log"), Some("file_writer"));
//!     let stdout_writer = system.actor_of(stdout_writer::props(), Some("stdout_writer"));
//!     let logger = system.actor_of(logger::props(file_writer.clone(), stdout_writer.clone()), Some("logger"));
//!     (logger, file_writer, stdout_writer)
//! };
//! ```
//! # More about DeadLetters
//! In the previous text will be described what is the DeadLetters. Now will be described why it
//! exists and how it use. Basic idea of the actors message passing, consists in that all message
//! must be delivered to the target actor and consumed. If this is not happens, and message not
//! consumed be the receiver, this is an semantic error in the your program. Main task of the
//! DeadLetters, is quite aggressive indicates to you, that this error type was occurs in your
//! program. For this, DeadLetter log each received by him message to the stdout.
//!
//! ```text
//! DeadLetter receive some message ... from 'ActorRef (/stdout_writer)' to 'ActorRef (/logger)'
//! ```
//!
//! This feature is unable to disable be the ideological reasons. Because not processed messages is
//! an error! In practical, this prints is very useful for actor system debug , when something goes
//! wrong and work improperly. 90% of errors in actor-based programs linked to the problem of not
//! delivered messages and DeadLetters allows to you fastly determine bags.
//!
//! Also, you may encounter with situation, when you may want to interact with DeadLetters directly.
//! This is may be performed by obtaining the DeadLetter actor reference. This reference stored in
//! the ActorSystem and may by cloned from it by the special function:
//!
//! ```
//! let mut deadLetters = system.dead_letters();
//! ```
//!
//! This is a regular actor reference, but all messages passed through it, will be dropped directly
//! to the DeadLetter.
//!
//! # Timers
//!
//! Timers is the separate module which is uses for scheduling automatic message sending with a
//! specified time-params. Main his purpose is sending messages from an actor to himself for
//! determine timeout of various asynchronous operations. Next example demonstrates contrived actor,
//! uses this features. Realization details was dropped for more clear understanding of the concept
//! ( you may found full code in the examples module ).
//!
//! ```
//! impl Actor for Ticker {
//!
//!    fn pre_start(self: &mut Self, ctx: ActorContext) {
//!        let mut timers = Timers::new(ctx.system.clone());
//!
//!        timers.start_single(
//!           0,
//!           &ctx.self_,
//!           &ctx.self_,
//!           Duration::from_secs(1),
//!           msg!(SingleTick {}));
//!
//!        timers.start_periodic(
//!            1,
//!            &ctx.self_,
//!            &ctx.self_,
//!            Duration::from_secs(2),
//!            || msg!(PeriodicTick {}));
//!
//!        self.timers = Some(tsafe!(timers));
//!    }
//!
//!    fn post_stop(&mut self, _ctx: ActorContext) {
//!        self.timers.as_ref().unwrap().lock().unwrap().cancel_all();
//!    }
//!
//!
//!    fn receive(self: &mut Self, msg: Message, ctx: ActorContext) -> bool {
//!        let msg = msg.get();
//!        match_downcast_ref!(msg, {
//!            m: SingleTick => {
//!               println!("SingleTick");
//!            },
//!            m: PeriodicTick => {
//!               if (self.ticks == 3) {
//!                   self.timers.as_ref().unwrap().lock().unwrap().cancel(1);
//!                   println!("PeriodicTick cancelled");
//!               } else {
//!                   println!("PeriodicTick");
//!                   self.ticks = self.ticks + 1;
//!              }
//!           },
//!            _ => return false
//!        });
//!
//!        true
//!    }
//! }
//! ```
//!
//! Here in the pre_start hook, creates two timers. All timers has practically identical set of
//! arguments. You must specify key, on which timer may be cancelled, sender and receiver refs, time
//! parameter and message for sending. First timer is the single timer, it is send message only once
//! with one second delay. Message are sent from self to self. As message specified SingleTick
//! structure. Second timer is the periodic timer. He will send message to the actor each two
//! second, until he will be explicitly cancelled. Message argument contains some trick. It's accept not
//! struct itself, but closure which produce this struct. It is due to than for each timer iteration,
//! schedulers needs the new message. If you send to this arg a structure, it will be fully consumed at
//! first iteration, and timer will does not have what to send. Closure used here as factory of a
//! single message.
//!
//! After all timers was configured, timer object stored in the actor state. Through one second
//! after start, the actor will receive the SingleTick message. After two, the actor will start
//! receive the periodic message - PeriodicTick. After receive three such messages, it will cancel
//! this timer specify the timer key in the cancel method.
//!
//! In post_stop hook, all timers must be canceled. If this is don't do, you risk encounter with
//! situation, when timer will send message to dead actor and it will be dropped to DeadLetters.
//! This situation may be more dangerous, if intervals timers exists. If don't cancel it after actor
//! was stopped, memory leaks occurs, because actor doesn't may be dropped because of him.
//!
//! # Watching
//!
//! Each actor have special set of internal events. When event from this set occurs, it's passed
//! to the internal event bus. Watching allows one actor to subscribe to another actor's events.
//! Most significant usege of this technique is tracking of terminating of watched actors. You may
//! subscribe actor for events of another actor in the following way:
//!
//! ```
//! ctx.system().watch(&ctx.self_, &self.target);
//! ```
//!
//! First argument of the watch function is the reference to actor which will be subscribed for
//! events of the actor specified in the second argument. After that subscriber, will be receive
//! special messages always when some events occurs with the target actor. For now exists next event
//! types:
//!
//! * Terminated - occurs when watched actor is fully stopped (event occurs right after post_stop
//! hook was called).
//!
//! Actor may unsubscribes from actor events in similar way, using function unwatch:
//!
//! ```
//! ctx.system().unwatch(&ctx.self_, &self.target);
//! ```
//!
//! Caution about unwatching. You must always explicitly call unwatch method, before actor (which is
//! watch to another actors) will be stopped. This is non critical, but if you don't do it, this
//! cause to full walk of event bus tree for search and remove stopped actor from subscribers of
//! events. It may be extremely expensive operation if on the event bus exists many observed actors.
//!
//! You may see full example usage of watching in the 'examples/actors/watch' submodule.
//!
//! # Ask
//!
//! Ask call doing the same thing that tell call, but expects, that target actor respond back with
//! some message. This expectation represents as Future, which will be completed with received
//! message or with AskTimeoutError if target actor does not respond with some timeout. This
//! operation may be used in three way:
//!
//! * Interact with actor system from synchronous world. This feature will be presented in the
//! nearest release of the futures. The essence of this patterns consists in that you Ask target,
//! and block your thread on the Future, which returned be the function. This operation will
//! permits to you interact with actors as if they would  be a some synchronous function.
//! * Connection of actor's world with future's world. This situation is very often occurs in the
//! web development. Most count of modern server libraries, more or less supports future paradigm.
//! This features consists in than when server receives a request, you do not process him in the
//! handler but pass this work to the future. With Ask pattern you may represents interaction
//! with actor system through such future. Basic example of this interactions may be expressed
//! in this way:
//!
//! ```
//! actor.ask(&mut (*system.lock().unwrap()), msg!(SomeRequest {}))
//!     .map(|v| {
//!         // Do some with received message
//!     })
//!     .recover(|e| {
//!         // Do some with timeout error
//!     });
//! ```
//! * Intercommunication layer between actors without mutation of state . Imagine the next station.
//! You have three actors A, B and C. A requests B for some data. But B for respond to A, need
//! firstly request C and based on him response, he may create response for A. Without futures this
//! may be realized only as state machine. After receives request from A, B sends request to C and
//! change it's behavior for handle responses from C. When response from C will be received,
//! B create response to A, send him, and switch her behavior to the normal. Details about what do
//! with messages addressed to B when it's not in the normal state is omitted. If in this situation
//! state of actor B does not mutates, state machine may be converted to actor with linear behavior,
//! which uses for this operation futures. Such code Ask actor C and does not wait anything, all
//! logic now live in the future combinators, which will be processed fully separate from the actor B.
//! Situation of this sort, often have a place to be in DDD paradigm in parts of CQRS interactions
//! logic. For example you ask to root aggregate for get some view object of some entity, he will
//! calls read side of entity, read side in turn asks repository. Repository return raw view object
//! to the read side. Read side transforms this object to the natural view object, and root
//! aggregate removes some system fields from it and returns it to the requester. All stages in this
//! chain is immutable for state of the participants. And all this operation may be performed with
//! futures. On practice it may work follows:
//!
//! ```
//! fn receive(&mut self, msg: Message, mut ctx: ActorContext) -> bool {
//!     let msg = msg.get();
//!     match_downcast_ref!(msg, {
//!         m: commands::RequestFromA => {
//!             let inter_data = m.data + 100;
//!
//!             let mut sender = ctx.sender.clone();
//!             let mut self_ = ctx.self_.clone();
//!
//!             self.c_actor.ask(&mut (*ctx.system()), msg!(RequestFromB { data: inter_data }) )
//!                 .on_complete(move |v| {
//!                     if v.is_ok() {
//!                         let resp = v.as_ref().ok().unwrap();
//!                         let resp = resp.get();
//!                         if let Some(m) = resp.downcast_ref::<ResponseFromC>() {
//!                             let inter = m.data;
//!                             sender.tell(msg!(ResponseFromB {data: inter }), Some(&self_));
//!                         }
//!                     } else {
//!                         println!("Oops! C does not respond!")
//!                     }
//!                 });
//!             },
//!             _ => return false
//!         });
//!
//!    true
//! }
//! ```
//!
//! You may see full example of Ask usage in the 'examples/actors/ask' submodule.
//!
#[macro_use] pub mod message;
pub mod prelude;
pub mod dispatcher;
pub mod default_dispatcher;
pub mod actor_cell;
pub mod envelope;
pub mod mailbox;
pub mod unbound_mailbox;
pub mod actor;
pub mod local_actor_system;
pub mod abstract_actor_ref;
pub mod props;
pub mod actor_path;
pub mod actor_context;
pub mod dead_letters;
pub mod synthetic_actor;
pub mod actor_ref_factory;
pub mod abstract_actor_system;
pub mod local_actor_ref;
pub mod scheduler;
pub mod timers;
pub mod watcher;
pub mod ask_actor;