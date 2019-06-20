//! Asynchronous completed promise
//!
//! This type of promise is designed to be filled with result produced from a function, execute on
//! executor. Passed closure will immediately planned for execution on specified executor. After
//! execution, result of function will automatically completes internal future.
//!
//! # Examples
//!
//! See the module level documentation.
//!

use crate::futures::promise::{Promise};
use crate::futures::future::{Future, WrappedFuture};
use crate::executors::executor::Executor;
use crate::common::tsafe::TSafe;
use std::sync::{Arc, Mutex};

pub struct AsyncPromise<V: Send + 'static, E:  Send + Clone + 'static> {
    pub future: TSafe<Future<V, E>>
}

impl <V: Send + Clone, E: Send + Clone> AsyncPromise<V, E> {
    pub fn new(f: Box<FnMut() -> Result<V, E> + Send>, executor: TSafe<Executor>) -> AsyncPromise<V, E> {
        let fut = tsafe!(Future::new());
        let mut f = f;
        let executor = executor;
        let mut obj = AsyncPromise {
            future: fut.clone()
        };

        executor.lock().unwrap().execute(Box::new(move || {
            let result = f();
            obj.complete(result);
        }), None);

        AsyncPromise {
            future: fut
        }
    }
}

impl <V: Send + Clone, E: Send + Clone> Promise<V, E> for AsyncPromise<V, E> {
    fn try_complete(&mut self, result: Result<V, E>) -> bool {
        if self.future.lock().unwrap().is_completed() {
            false
        } else {
            self.future.lock().unwrap().complete(result);
            true
        }

    }

    fn future(&self) -> WrappedFuture<V, E> {
        WrappedFuture {inner: self.future.clone()}
    }
}

