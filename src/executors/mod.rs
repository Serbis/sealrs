//! Executors environment
//!
//! # Table of Contents
//! 1. [Introduction](#introduction)
//! 2. [ThreadPinnedExecutor](#threadpinnedexecutor)
//!
//! # Introduction
//!
//! In modern asynchronous frameworks, executors is a second level of architecture after threads of
//! operation system. This objects allows pass some task to them and start to do some other work,
//! right after that. How and when this work will be really executed, defined by an executor
//! implementation. This async primitive is a basic on which developed primitives of next
//! architecture level - actors and promises. Actors uses executors as basis for message dispatching
//! system. Promises uses they as internal runtime environment.
//!
//! Technically executor is a trait with the single abstract method:
//!
//! ```
//! fn execute(&mut self, f: ExecutorTask, options: Option<Box<Any>>);
//! ```
//!
//! Besides on this, actors and futures is fully indifferent about how executors work internal, and
//! they know nothing about this. They see only the 'execute' method with hidden realization. This
//! fact allows to create any type of executors and slip it them without any impacts to code of the
//! original module. You may to use executors from predefined set or you may implement your own
//! version based on the 'Executor' trait. For now, in the library exists few predefined versions
//! of executors.
//!
//! # ThreadPinnedExecutor
//!
//! This executor creates thread pool with specified counts of threads. Each thread has it's own
//! queue. When you pass task to the executor, you may explicitly specify thread id on which
//! this task will be executed. If thread id does not specified explicitly, it will be selected
//! automatically bases on the task distribution strategy. You may use this executor type by follow:
//!
//! ```
//! let mut executor = ThreadPinnedExecutor::new()
//!     .set_threads_count(8)
//!     .set_distribution_strategy(DistributionStrategy::Load)
//!     .run();
//!
//! let f0 = Box::new( || { println!("Task on implicitly selected thread") });
//! executor.execute(f0, None);
//!
//! let f1 = Box::new( || { println!("Task on explicitly selected thread with id 6") });
//! executor.execute(f1, Some( Box::new(TaskOptions { thread_id: Some(6) } )));
//! ```

pub mod executor;
pub mod thread_pinned_executor;