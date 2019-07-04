//! Error wrapper
//!
//! This object used as main unit of error passing interaction. He may contain any data type and
//! provides he safe shared between threads and concurrent access. This object may be cloned
//! unlimited number of times.
use crate::common::tsafe::TSafe;
use std::any::Any;
use std::sync::{MutexGuard};

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
macro_rules! err {
    ($l:expr) => {
        {
           Error::new(tsafe!($l))
        }
    };
}


pub struct Error {

    /// Wrapped data
    pub inner: TSafe<Any + Send>
}

impl Error {
    pub fn new(inner: TSafe<Any + Send>) -> Error {
        Error {
            inner
        }
    }

    /// Returns MutexGuard of an inner error
    pub fn get(&self) -> MutexGuard<Any + Send> {
        self.inner.lock().unwrap()
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Error {
            inner: self.inner.clone()
        }
    }
}