//! Message wrapper
//!
//! This object used as main unit of message passing interaction. He may contain any data type and
//! provides he safe shared between threads and concurrent access. This object may be cloned
//! unlimited number of times.
use crate::common::tsafe::TSafe;
use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard};

/// Wraps any data type to the message wrapper
///
/// # Example
///
/// ```
/// msg!(10);
/// msg!(String::from("xxx");
/// msg!(SomeStructure { });
/// ```
///
#[macro_export]
macro_rules! msg {
    ($l:expr) => {
        {
           Message::new(tsafe!($l))
        }
    };
}


pub struct Message {

    /// Wrapped data
    pub inner: TSafe<Any + Send>
}

impl Message {
    pub fn new(inner: TSafe<Any + Send>) -> Message {
        Message {
            inner
        }
    }

    /// Returns MutexGuard of an inner message
    pub fn get(&self) -> MutexGuard<Any + Send> {
        self.inner.lock().unwrap()
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            inner: self.inner.clone()
        }
    }
}