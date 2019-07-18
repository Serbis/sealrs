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
//! 10. [More about dispatchers](#more-about-dispatchers)
//! 11. [Timers](#timers)
//! 12. [Stash](#stash)
//! 13. [Watching](#watching)
//! 14. [Ask](#ask)
//! 15. [FSM](#fsm)
//! 16. [Supervision](#supervision)
//! 17. [Remoting](#remoting)
//!
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
//! * Various dispatchers realizations
//! * FSM
//!
//! # Perspective features:
//! Under this features already exists architectural basis, and their implementation is a question
//! of my free time.
//!
//! * Mailboxes with user defined functional
//! * Supervising
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
//!     * DefaultDispatcher - dispatcher based on on ThradPinnedDispatcher with thread pinning
//! stratagy. This executor contants set of threads, which distributes between actors. He is used
//! for main purpose and non blocking tasks, with medium loading for the system.
//!     * PinnedDispatcher - dispatcher which contain only one thread and service work of only one
//! actor. Stops at the same time with serviced actor. Used for situations when actor needs to
//! perform some heavy synchronous tasks such as working with a files or using a network. Blocking
//! of message processing in this dispatcher have nothing impact to other actors in the system.
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
//!     fn receive(self: &mut Self, msg: Message, _ctx: ActorContext) -> HandleResult {
//!         let msg = msg.get();
//!         match_downcast_ref!(msg, {
//!             m: Print => {
//!                 self.printed_chars = self.printed_chars + m.text.len();
//!                 println!("{}", m.text);
//!             },
//!             _ => return Ok(false)
//!         });
//!
//!         Ok(true)
//!     }
//! }
//! ```
//!
//! Next we write the code which use this actor:
//!
//! ```
//! let mut system = LocalActorSystem::new();
//!
//! let mut printer = system
//!     .actor_of(basic_actor::props(), Some("printer"));
//!
//! let msg = msg!(basic_actor::Print { text: String::from("Hello world!") });
//! printer.tell(msg, None);
//! ```
//!
//! At first line, we create new actor system. Internally, this operation creates default dispatcher
//! and DeadLetter synthetic actor. At next line, we create the early defined actor. actor_of
//! functions is receive Pops object as
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
//! called. This function must return a HandleResult value, which indicates fact, was  handled message or
//! not (or some error occurs). If the message was handled, message processing logic is end, else the message will be dropped
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
//! * pre_start - called after full actor initialization and before allowing reception of
//! messages. At this stage already exists actor context, which may be used for creating new actors
//! or sending message for himself. Also is available actor's internal state for mutating.
//! * post_stop - called after full stop of actor. This means, that all message passed to the actor
//! was processed or dropped and it will not receive any new messages further. This is the last
//! point of interaction with internal actor's state. At this stage actor context also exists and
//! may be user to perform various operations.
//! * pre_fail - Called before some supervising operation will be started. Read more about what it
//! in the special chapter.
//! * post_restart - Called after actor was restarted. This may occurs if some supervision action
//! occurs. Read more about what it in the special chapter.
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
//!     system.stop(bench);
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
//!fn receive(self: &mut Self, msg: Message, ctx: ActorContext) -> HandleResult {
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
//!            _ => return Ok(false)
//!        });
//!
//!    Ok(true)
//!}
//!
//!
//!//StdoutWriter
//!fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> HandleResult {
//!    let msg = msg.get();
//!    match_downcast_ref!(msg, {
//!            m: Write => {
//!               println!("{}", m.text);
//!               let resp = msg!(Ok { chars_count: m.text.len() });
//!               ctx.sender.tell(resp, Some(&ctx.self_));
//!            },
//!            _ => return Ok(false)
//!        });
//!
//!    Ok(true)
//!}
//!
//!
//! //FileWriter
//!fn receive(self: &mut Self, msg: Message, mut ctx: ActorContext) -> HandleResult {
//!    let msg = msg.get();
//!    match_downcast_ref!(msg, {
//!            m: Write => {
//!               fs::write(&self.file, m.text.as_bytes());
//!               let resp = msg!(Ok { chars_count: m.text.len() });
//!               ctx.sender.tell(resp, Some(&ctx.self_));
//!            },
//!            _ => return Ok(false)
//!        });
//!
//!    Ok(true)
//!}
//! ```
//!
//! And code which use this actors:
//!
//! ```
//! let mut system = LocalActorSystem::new();
//!
//! let mut logger =  {
//!     let mut system = system;
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
//!     let mut system = system;
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
//! # More about dispatchers
//!
//! Messages dispatch is more complex topic for describe him in one paragraph. Next text
//! completes this omission.
//!
//! ## Specifying the dispatcher via props
//!
//! You may specify dispatcher for actor by presents her name to the Props object. By default,
//! exists two dispatchers type - default and pinned. Default used alwasy as default if you don't
//! set another type explicitly. You may specify other dispatcher by calling with_dispatcher
//! function.
//!
//! ```
//! let some_actor = Props::new(tsafe!(SomeActor::new()))
//!     .with_dispatcher("pinned");
//! ```
//!
//! ## Adding new dispatchers
//!
//! You can add external dispatchers to the actor system through add_ dispatcher method. This method
//! receive name of created dispatcher and his object. Should notice that if dispatcher with the
//! same name exists, it will be replaced. Old dispatcher will be immediately stoppd and dropped.
//! After adding dispatcher, he may be used with Props object through with_dispatcher method.
//! Replacing feature, permits you to replace default dispatcher of the actor system.
//!
//! ```
//!
//! // Replace default dispatcher with custom count of threads
//! system.add_dispatcher("default", tsafe!(DefaultDispatcher::new(16)));
//!
//! ```
//!
//! ## Implementing custom dispatcher
//! You can implement your own dispatcher type with some specific functionality. This is not magic
//! action, because actor system dispatcher is a simple struct which implements the two traits -
//! Dispatcher and Executor. You may implement this struct and pass it with add_dispatcher method to
//! the actor system. This action is contain too many code for describe him in this doc. You may see
//! to the example of custom dispatcher implementing in the 'example/actors/custom_dispatcher'.
//!
//! ## Using dispatchers with futures
//!
//! Fact that dispatcher implements Executor trait, allows to you use him as execution context of
//! AsyncPromise. You can do this in such way:
//!
//! ```
//!
//! ```
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
//!        let mut timers = RealTimers::new(ctx.system.clone());
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
//!            Box::new(|| msg!(PeriodicTick {})));
//!
//!        self.timers = Some(tsafe!(timers));
//!    }
//!
//!    fn post_stop(&mut self, _ctx: ActorContext) {
//!        self.timers.cancel_all();
//!    }
//!
//!
//!    fn receive(self: &mut Self, msg: Message, ctx: ActorContext) -> HandleResult {
//!        let msg = msg.get();
//!        match_downcast_ref!(msg, {
//!            m: SingleTick => {
//!               println!("SingleTick");
//!            },
//!            m: PeriodicTick => {
//!               if (self.ticks == 3) {
//!                   self.timers.cancel(1);
//!                   println!("PeriodicTick cancelled");
//!               } else {
//!                   println!("PeriodicTick");
//!                   self.ticks = self.ticks + 1;
//!              }
//!           },
//!            _ => return Ok(false)
//!        });
//!
//!        Ok(true)
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
//! # Stash
//!
//! If an actor have several lines of behavior, exists one serious problem. What to do with messages
//! not from current behavior? For this situation, exists special pattern named stashing. He
//! expressed as simple struct which works as user side mailbox. You can put messages into it and when
//! need you may initiate resend of stored messages. Normally stash created in pre_start hook:
//!
//! ```
//! self.stash = RealStash::new(&ctx);
//! ```
//!
//! And after that you can use it:
//!
//! ```
//! self.stash.stash(&msg, &ctx);
//!
//! // In other message handler ...
//!
//! self.stash.unstash_all();
//! ```
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
//! * Interact with actor system from synchronous world. The essence of this patterns consists in
//! that you Ask target, and block your thread on the Future, which returned be the function. This
//! operation will permits to you interact with actors as if they would  be a some synchronous
//! function. This operation must be never used from the actor system, because it will cause to block
//! of the dispatcher, that will very fast leads to dead lock of the entire system.
//!
//! ```
//! let req = actor.ask(&mut (system), msg!(SomeRequest {}));
//! let result = req.result(Duration::from_secs(5));
//!
//! // Do some actions with received message or handle error
//! ```
//! * Connection of actor's world with future's world. This situation is very often occurs in the
//! web development. Most count of modern server libraries, more or less supports future paradigm.
//! This features consists in than when server receives a request, you do not process him in the
//! handler but pass this work to the future. With Ask pattern you may represents interaction
//! with actor system through such future. Basic example of this interactions may be expressed
//! in this way:
//!
//! ```
//! actor.ask(&mut (system), msg!(SomeRequest {}))
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
//! fn receive(&mut self, msg: Message, mut ctx: ActorContext) -> HandlerResult {
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
//!             _ => return Ok(false)
//!         });
//!
//!    Ok(true)
//! }
//! ```
//!
//! You may see full example of Ask usage in the 'examples/actors/ask' submodule.
//!
//! # FSM
//!
//! FSM is transcripted as Finite State Machine. This conceptions is very powerful used with
//! actors, because permits you to create fully asynchronous state machines with full control
//! around them behavior.
//!
//! Actor may be presented as FSM if it her logic satisfies few things:
//! * He uses few lines of behavior with various data
//! * Some behaviours needs a timeouts
//! * Lifetime of an actor is finite (this is partial demand, because may exist fsm with lifetime
//! equals to lifetime of entire program).
//!
//! All this aspects may be realized with conditional operators and timers, but in this case, your
//! code will be is very dirty. For eliminate this difficulties, in the library exists special
//! module, which contains abstraction layer for creating of FSM.
//!
//! ### State and Data
//!
//! State is the main executable unit of any FSM. Each state accompanied by a some Data. In any
//! piece of time, FSM always be in one of some State and have one of some Data. States and Data
//! on the language level describes as enum. Separate State handlers describes as function with the next
//! signature:
//!
//! ```
//! pub fn state0(&mut self, msg: &Message, ctx: &ActorContext, data: &Data) -> StateResult<Self, State, Data> {
//!     //Make a some actions with a message based on a presented Data
//! }
//! ```
//! You may perceive this handler as special variant of the standard receive function of actor.
//! There you may do similar actions as in the receive functions, buf instead HandleResult, you
//! must return StateResult. Last type is the alias for the very complex type, so for prevent you from
//! creating a trash code, library presents few functions for produce this result. This result
//! indicates to the upper library code, what to do next - switch state or data, stop actor or
//! some other things.
//!
//! * Fsm::goto(state) - indicates to change current state to some other.
//! * Fsm::goto_using(state, data) - indicates to change current state and data to some others
//! * Fsm::stay() - indicates to stay in current state and with current data
//! * Fsm::stay_using(data) - indicates to stay in current state but with some other data
//! * Fsm::stop() - indicates that work of FSM is completed (stops the actor)
//! * Fsm::unhandled() - indicates that current message was doest not be handled (unknown type
//! of message or some other reasons)
//!
//! Each state handlers must be explicitly registered in the fsm object:
//!
//! ```
//! fsm.register_handler(State::StateA, Self::state_a, Duration::from_secs(5));
//! ```
//!
//! In this call you must specify the state on which attached handler, handler function and state
//! timeout.
//!
//! ### States timeouts
//!
//! Each separate state have it's own idle timeout. If fsm will does not receive any messages in
//! this timeout, library automatically send special message SendTimeout, which will be indicates
//! to the user code, that state timeout was reached. What to do next in this situation is youre
//! decision.
//!
//! ### Unhandled messages
//!
//! You may register special handler of unhandled messages. This function will be called always,
//! when some state completes with Fsm::unhandled(). Handler is useful when you need detect
//! unhandled message in state independent manner. Handler presented as next function:
//!
//! ```
//! fn unhandled(&mut self, msg: &Message, ctx: &ActorContext, state: &State, data: &Data) -> HandleResult
//! ```
//!
//! Signature is analogous for default receive function of an actor, but contains additional
//! information about state and data, under which fsm was missed a message. Handler may be
//! registered through this method:
//!
//! ```
//! fsm.register_unhandled(Self::unhandled);
//! ```
//!
//! ### State transitions
//!
//! You may execute some code in the moment when fsm will be change status. Handler of this
//! situation have the next view:
//!
//! ```
//! fn transition(&mut self, from: &State, to: &State)
//! ```
//!
//! He contains in arguments link to the state from which fsm passes and a link to the state to
//! which fsm goes. This handler may be registered with the next call:
//!
//! ```
//! fsm.register_transition(Self::transition);
//! ```
//!
//! ### Full example
//!
//! You can see the full example of how to use fsm in 'examples/actor/fsm'
//!
//! ### When you need to use FSM and when Futures?
//!
//! When you start a real world work with fsm, you may encounter with the strange question - why
//! I need to use the fsm, if i may instead I use combined futures? I suffered from this issue
//! many cont of times, but in the end, I was create some conception of when user fsm and when use
//! futures. Here she is.
//!
//! Future - used when operation contains only lenear logic, less than three stage and it does not
//! need information about states. Example - you need checks user permissions and based on this
//! check, to request data from repository or not. All simple, two stage, no data between them and
//! no branching.
//!
//! Fsm - used when operation shares data between states or she contains more than three stages or
//! operation logic is non linear. Example. At first stage you request of authorisation data from the
//! local caсhe. If he contains data about user, you go to stage three. If not, you start stage two -
//! checking user permissions on a remote server. At stage thee you have user permissions check
//! result. If it is privileged user, you requests to repository for getting version of data for
//! privileged user. If not, you requests to repository for getting version of data for non-privileged
//! user. At stage four, based on this data from the repository, you respond to the originator with
//! some final result. Four stages, shared data between them and several scenarios.
//!
//! # Supervision
//!
//! All actors in the system grouped to tree, in which each actor have one parent and zero or more
//! children's. Supervision in the actor system relies on the two simple ideas. First idea consists
//! is that lifetime of all child actors directly depends on lifetime of his parents. Second idea
//! is that actor may be failed when process some message, and after that, system will take some
//! actions for restore broken actor from this situation.
//!
//! ### Hierarchy
//!
//! As was said before, each actor have one parent and may have childs. At the top of all hierarchy
//! placed Root Guardian - synthetic actor whose only task is to be parents for all actors which will
//! be created through system.actor_of method. You may create an actor through another way:
//!
//! ```
//! ctx.actor_of(some_actor::props(), Some("name"));
//! ```
//!
//! In this code as ActorRef provider ActorContext is used. Actor from called this action, becomes
//! as parent for created actor and his lifetime is fully depended on lifetime of his parent. How it
//! work on practice. For example you have the next actors hierarchy - /root/A/B/C. If you will stop
//! the A actor, it will cause to recursive stopping of actors B and C. What about root? Yes, he also
//! may be stopped. This action occurs when you call system.terminate method. Before the handler will
//! stops the dispatchers, he will stop the Root Guardian, what cause to stops of all actors in the
//! hierarchy.
//!
//! ### actor_select
//!
//! Hierarchy of actors gives us some useful feature. You may obtain ActorRef's of actors placed
//! in some leaf of the tree if you know them path.
//!
//! ```
//! let selection = system.actor_select("/root/a/b");
//! let b_ref = &mut selection[0];
//! ```
//!
//! Call of acotor_select return the vector with found ActorRef's. This is the regular refs which will
//! may be used as refs created through actor_of call.
//!
//! Need make a note, that actor_of is the some cost operation, because with searching of actor,
//! code goes through all hierarchy of cells, what cause to locks. Based on that, what then
//! deeper the target actor, than more expensive the search.
//!
//! ### Error recovery
//!
//! Message handler may completes with some error:
//!
//! ```
//! fn receive(self: &mut Self, msg: Message, _ctx: ActorContext) -> HandleResult {
//!     let msg = msg.get();
//!     match_downcast_ref!(msg, {
//!             m: HelloToC => {
//!                 println!("Hello to some from the outside world!")
//!             },
//!             m: FailMe => {
//!                 println!("CActor receive FailMe");
//!                 return Err(err!("some err"))
//!             },
//!             _ => return Ok(false)
//!         });
//!
//!     Ok(true)
//! }
//! ```
//!
//! This situation treated be system as signal that processed actor was faced with an insoluble
//! error, and system must take some actions. This actions is called supervision operation and
//! determined with supervision strategy. Exist four strategies:
//!
//! * Resume - do nothing, ignore occurred error
//! * Stop - stop the actor
//! * Restart - restart the actor (stop and start)
//! * Escalate - lift this error to the parent of the actor
//!
//! All created actors has Restart strategy by default. Root Guardian and DeadLetters has Resume
//! strategy. Strategy of a separate actor may be set up through Props object:
//!
//! ```
//! let props = Props::new(tsafe!(SomeActor::new()))
//!        .with_supervision_strategy(SupervisionStrategy::Escalate)
//! ```
//!
//! Before starts of the supervision operation, pre_fail hook of the failed actor will be called.
//! This hook contains the actor context, occurred error and current supervision strategy.
//!
//! # Remoting
//!
//! Remoting is the feature which concludes in opportunity of interaction between two actor systems
//! through network. This systems may be blaced in one application, in separate applications on one
//! or different physical machines. For now, realized next interaction features:
//!
//! * actor_select of actors on a remote actor system
//! * message sending between actor systems
//!
//! Remoting is concluded in few concepts, described below.
//!
//! ## RemoteActorRef
//!
//! When you work with ordinary actor system, you will always interact with LocalActorRef. This
//! type of refs works with internal actor representation in the system (actor cell). Whenever
//! you will work with some type of remote systems, together with LocalActorRef you will be
//! interact with RemoteActorRef. RemoteActorRef links instead of linked to cell, referenced
//! to network connection in the system network controller. Practical work with this type of
//! refs, no different from work with LocalActorRef, excluding some moments. You may read about
//! them in the Delivery Guarantee paragraph.
//!
//! ## NetworkActorSystem
//!
//! This is a complete of LocalActorSystem clone with one small exception - she have controller
//! of network connections which allow for other actor systems establish connection with her
//! through network. Constructor of the system, receives the two additional arguments - bind
//! address and link to a messages serializer. Bind address is a string which may be converted
//! to the SocketAddr object. About a messages serializer you may read below.
//!
//! ## RemoteActorSystem
//!
//! This is the thin wrapper over network controller which allows you to interact with a remote
//! actor system. You may think about her as domain oriented socket. She contains the same methods
//! like her big sisters (Remote/LocalActorSystem), but actions which do this methods will be
//! addressed to a remote system behind network connection. Constructor of this type of system
//! receives three additional arguments - connection address of a remote system, messages
//! serializer and host actor system. This system is always relies on a big actor system - local,
//! remote or test actor system.
//!
//! ## MessageSerializer
//!
//! Since Rust, unlike platforms such as JVM, does not contain developed means of language
//! reflection, you should explain to the library how to convert various messages into a
//! byte representation and back. For this target, library present the special trait -
//! MessageSerializer. Before moving on to further descriptions, should make the following
//! remark. Library does not determine how you will be serialized finite message. You may use
//! any type of serialization which will you like (json, protobuf, you own and etc).
//! MessageSerializer is the trait with two mandatory methods - to_binary and from_binary.
//! First method is receives a Message and must returns result with SerializationResult
//! or SerializationError struct. SerializationResult have two fields - marker and blob. Field
//! 'marker' is an unique number which can uniquely indicate this type of message. Field 'blob'
//! must contain binary representation of an original message.  How you transform original message
//! to binary array is your choice. SerializationError represents set of the standard failure
//! situations, which may be occurs at this operation.
//!
//! The method from_binary is a mirror for the to_binary method. He receives marker number and
//! blob and must return result with the Message or SerializationResult struct.
//!
//! ## Delivery guaranty
//!
//! When you work with local system, exists guarantee that a message passed to an actor will
//! reach the recipient (of course if he was not accidentally stopped ). When working with remote
//! system, this rule is does not work. It doesn't work, because many of thing may goes wrong -
//! connection may be lost, may occurs routing error,  low quality level of transmission on physical
//! level and many other bad things. In other words, passed message may does not reach the target
//! and you should always keep this in mind. You must constructs remote services in the way that
//! requests always have a reply message, which allows for you fixed the fact that message was
//! really delivered to the target.
//!
//! Also need pay attention to that fact, that actor behind RemoteActorRef may be stopped, and you
//! never know about it if you doesn't explicitly watch him.
//!
//! ## Blocking operations
//!
//! All method calls on RemoteActorRef, NetworkActorSystem and RemoteActorSystem have blocking IO
//! operation nature. In other words, this calls may lock dispatcher thread if you will be call's
//! them directly from an actor messages handler. As a consequence of this, exist only three places
//! where you may invoke them safely - outside of the actor system space, in async future and in
//! actor with pinned dispatcher.
//!
//! ## Practical usage
//!
//! How this technique may be used in practie you may see in 'examples/actor/remote'. First see to
//! the mod.rs for to understand how this example works.
//!
#[macro_use] pub mod message;
#[macro_use] pub mod error;
pub mod prelude;
pub mod dispatcher;
pub mod default_dispatcher;
pub mod pinned_dispatcher;
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
pub mod wrapped_dispatcher;
pub mod stash;
pub mod fsm;
pub mod supervision;
pub mod remoting;
