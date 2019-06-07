//! 'One queue per thread' implementation of executor
//!
//! In this implementation of executor, each thread has its own task queue. Caller code may
//! explicitly specify thread number on which task will be executed. If thread number does not set,
//! thread selection strategy will be used. Should pay attention on that selection of thread number
//! by the executor is always slowly than his explicitly setting, because thread selecting algorithm
//! is always perform some more heavy work, compared to direct task loading.
//!
//! # Examples
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

use crate::executors::executor::{Executor, ExecutorTask};
use crate::common::tsafe::TSafe;
use std::sync::{Mutex, Arc, Condvar};
use std::any::Any;
use std::collections::vec_deque::VecDeque;
use std::thread;
use std::time::Duration;
use rand::{Rng};

/// Thread queue type alias
type Queue = TSafe<VecDeque<ExecutorTask>>;

/// Automatically thread selection strategy
#[derive(Clone)]
pub enum DistributionStrategy {
    /// Uses thread_rng for generate random id of thread
    Random,

    /// Uses round-robin counter for selecting thread id
    Round,

    /// Selects less loaded thread
    Load,

    /// Uses always single thread from the pool (id = 0)
    EventLoop
}

/// Options which will may be passed with task
pub struct TaskOptions {

    /// Thread id on which task will be executed. Attention, out of range thread id will cause to
    /// panic!
    pub thread_id: Option<usize>
}

pub struct ThreadPinnedExecutor {
    /// Total threads count
    threads_count: usize,

    /// Implicit thread id selecting strategy
    distribution_strategy: DistributionStrategy,

    /// Threads queues
    queues: Vec<Queue>,

    /// Threads locks
    locks: Vec<Arc<Condvar>>,

    /// Threads stop flags
    stops: Vec<TSafe<bool>>,

    /// Rounds counter for the Round distribution strategy
    rounds: usize
}

impl ThreadPinnedExecutor {

    /// Creates new executor with default settings. Thread counts = count of the physical cpu
    /// core's. Distribution strategy = Round.
    pub fn new() -> ThreadPinnedExecutor {
        let cpu_count = num_cpus::get();
        ThreadPinnedExecutor {
            threads_count: cpu_count,
            distribution_strategy: DistributionStrategy::Round,
            locks: Vec::new(),
            queues: Vec::new(),
            stops: Vec::new(),
            rounds: 0
        }
    }

    /// Set thread's count
    pub fn set_threads_count(mut self, count: usize) -> Self {
        self.threads_count = count;
        self
    }

    /// Set distribution strategy
    pub fn set_distribution_strategy(mut self, strategy: DistributionStrategy) -> Self {
        self.distribution_strategy = strategy;
        self
    }

    /// Starts executor threads
    pub fn run(mut self) -> Self {

        for i in 0..self.threads_count {
            let stop = tsafe!(false);
            let queue = tsafe!(VecDeque::new());
            let cvar = Arc::new(Condvar::new());
            let mutex = Mutex::new(false);
            let _tid = i;

            self.queues.push(queue.clone());
            self.locks.push(cvar.clone());
            self.stops.push(stop.clone());

            thread::spawn(move || {
                while *stop.lock().unwrap() == false {
                    let f: Option<ExecutorTask> = {
                        let mut q = queue.lock().unwrap();
                        if q.len() > 0 {
                            Some(q.pop_front().unwrap())
                        } else {
                            None
                        }
                    };

                    if f.is_some() {
                        f.unwrap()();
                    } else {
                        cvar.wait_timeout(mutex.lock().unwrap(), Duration::from_millis(1000));
                    }
                }
            });
        }

        self

    }

    /// Stops executor threads
    pub fn stop(mut self) {
        for stop in self.stops.iter() {
            *stop.lock().unwrap() = true;
        }

        for cvar in self.locks.iter() {
            cvar.notify_all();
        }
    }

    /// Realizes implicit thread id selecting based on specified execution strategy
    fn get_thread_id(&mut self, strategy: DistributionStrategy) -> usize {
        match strategy {
            DistributionStrategy::Load => {
                let mut min = 1000000000;
                let mut min_q = 0;
                let mut qn = 0;

                for q in self.queues.iter() {
                    let len = q.lock().unwrap().len();
                    if len < min {
                        min = len;
                        min_q = qn;
                    }
                    qn = qn + 1;
                }

                min_q
            },
            DistributionStrategy::Round => {
                if self.rounds == self.threads_count - 1 {
                    self.rounds = 0;
                } else {
                    self.rounds = self.rounds + 1;
                }

                self.rounds
            },
            DistributionStrategy::Random => {
                rand::thread_rng().gen_range(0, self.threads_count - 1)
            },
            DistributionStrategy::EventLoop => {
                0
            }
        }
    }
}

impl Executor for ThreadPinnedExecutor {

    /// Plans task for execution. This method must be called only after the executor is running.
    /// Otherwise it will cause to panic!
    fn execute(&mut self, f: ExecutorTask, options: Option<Box<Any>>) {
        let thread_id = if options.is_some() {
            let options = options.unwrap();
            let options = options.downcast_ref::<TaskOptions>().unwrap();

            if options.thread_id.is_some() {
                options.thread_id.unwrap()
            } else {
                self.get_thread_id(self.distribution_strategy.clone())
            }
        } else {
           self.get_thread_id(self.distribution_strategy.clone())
        };

        self.queues[thread_id].lock().unwrap().push_back(f);
        self.locks[thread_id].notify_one();

    }
}