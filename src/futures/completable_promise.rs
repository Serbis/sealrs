//! Manually completable promise
//!
//! This type of promise is intended to be filled from the application code, which should call
//! the success or failure methods on it. After that, the passed value will be placed in the future.
//!
//! # Examples
//!
//! See the module level documentation.
//!

use crate::futures::promise::{Promise};
use crate::futures::future::{Future, WrappedFuture};
use crate::common::tsafe::TSafe;
use std::sync::{Arc, Mutex};

pub struct CompletablePromise<V: Send + Clone + 'static, E: Send + Clone + 'static> {
    pub future: TSafe<Future<V, E>>
}

impl <V: Send + Clone, E: Send + Clone> CompletablePromise<V, E> {
    pub fn new() -> CompletablePromise<V, E> {
        CompletablePromise {
            future: tsafe!(Future::new())
        }
    }
}

impl <V: Send + Clone, E: Send + Clone> Promise<V, E> for CompletablePromise<V, E> {
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