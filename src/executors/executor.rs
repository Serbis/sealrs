//! Basic trait for all executors

use std::any::Any;

pub type ExecutorTask = Box<FnMut() -> () + Send>;

pub trait Executor {
    fn execute(&mut self, f: ExecutorTask, options: Option<Box<Any>>);
    fn stop(&mut self);
}