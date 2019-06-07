//! Abstract promise definition
//!
//! This trait defines the concept of future / promise in its classic presentation.
//! According to it, the future is a synchronous container for the result, and a promise is some
//! function that fills the future. At the same time, the properties of this function are not
//! initially defined, therefore this base class is presented in the form of abstract interfaces.
//! The concrete implementation of the promise determines how the function will be
//! executed - synchronous, asynchronously or otherwise. For more detailed descriptions of what
//! specific promise does as well as the logic of the future, see the corresponding implementations.

use super::future::{Future, WrappedFuture};
use crate::common::tsafe::TSafe;
use std::sync::{Arc, Mutex};


pub trait Promise<V: Send + Clone, E: Send + Clone> {

    /// Implementation of this method must try to complete future
    fn try_complete(&mut self, result: Result<V, E>) -> bool;

    /// Completes promise with some result. Attention, promise may be completed only once and
    /// repeat attempt to do this will cause to panic!
    fn complete(&mut self, result: Result<V, E>) -> bool {
        if self.try_complete(result) {
            true
        } else {
            panic!("Promise already completed!");
        }
    }

    /// Completes promise with success result
    fn success(&mut self, value: V) -> bool {
        self.complete(Ok(value))
    }

    /// Completes promise with failed result
    fn failure(&mut self, cause: E) -> bool {
        self.complete(Err(cause))
    }

    /// Returns copy of the inner future as wrapper object
    fn future(&self) -> WrappedFuture<V, E>;
}