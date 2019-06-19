//! Wrapper for dispatcher which permits to him mimic to an executor

use crate::common::tsafe::TSafe;
use crate::actors::dispatcher::Dispatcher;
use crate::executors::executor::{Executor, ExecutorTask};
use std::any::Any;

pub struct WrappedDispatcher {
    dispatcher: TSafe<Dispatcher + Send>
}

impl WrappedDispatcher {
    pub fn new(dispatcher: TSafe<Dispatcher + Send>) -> WrappedDispatcher {
        WrappedDispatcher {
            dispatcher
        }
    }
}

impl Executor for WrappedDispatcher {
    fn execute(&mut self, f: ExecutorTask, options: Option<Box<Any>>) {
        self.dispatcher.lock().unwrap().execute(f, options)
    }

    fn stop(&mut self) {
        self.dispatcher.lock().unwrap().stop()
    }
}