//! Container for value which while does not calculated
//!
//! A container for the result of completing some promise. Another words, this is a container
//! for result that will be sometime later. It is possible to attach various functional combinators
//! to this container, which will be executed immediately after receiving the result from the
//! promise.

use super::completable_promise::CompletablePromise;
use super::promise::Promise;
use super::async_promise::AsyncPromise;
use crate::common::tsafe::TSafe;
use crate::executors::executor::Executor;
use std::sync::{Arc, Mutex};

/// Wrapper for future. This object encapsulate original future and allows to user set of
/// simplified methods mirrored from the original future (read as methods with sweetened syntax).
pub struct WrappedFuture<V: Send  + Clone + 'static, E: Send + Clone + 'static> {
    pub inner: TSafe<Future<V, E>>
}

impl <V: Send + Clone, E: Send + Clone> WrappedFuture<V, E> {

    /// Callback that will be called upon completion of the future. Creates and returns new future,
    /// which value is the result of function f applied to the current value if it is has 'Ok' type.
    /// Else function f does not applied, and current value inserted to the next future as is.
    ///
    /// # Examples
    ///
    /// See the module level documentation.
    ///
    pub fn map<S, F>(&mut self, mut f: F) -> WrappedFuture<S, E>
        where S:  Send + Clone + 'static,
              F: FnMut(&V) -> Result<S, E> + Send + 'static
    {
        self.inner.lock().unwrap().map(Box::new(f))
    }

    /// Callback that will be called upon completion of the future. Creates and returns new future,
    /// which value is the result of function f applied to the current value if it is has 'Ok' type.
    /// Else function f does not applied, and current value inserted to the next future as is.
    ///
    /// # Examples
    ///
    /// See the module level documentation.
    ///
    pub fn recover<F>(&mut self, mut f: F) -> WrappedFuture<V, E>
        where F: FnMut(&E) -> Result<V, E> + Send + 'static
    {
        self.inner.lock().unwrap().recover(Box::new(f))
    }

    /// Callback that will be called upon completion of the future. Creates and returns new future,
    /// which value is the result of completion of the future returned by the function f.
    ///
    /// # Examples
    ///
    /// See the module level documentation.
    ///
    pub fn flat_map<S, F>(&mut self, mut f: F) -> WrappedFuture<S, E>
        where S: Send + Clone + 'static,
              F: FnMut(&V) -> Result<WrappedFuture<S, E>, E> + Send + 'static
    {
        self.inner.lock().unwrap().flat_map(Box::new(f))
    }

    /// Callback that will be called upon completion of the future. Passes the future value as a
    /// function argument.
    ///
    /// # Examples
    ///
    /// See the module level documentation.
    ///
    pub fn on_complete<F>(&mut self, mut f: F)
        where F:FnMut(&Result<V, E>) -> () + Send + 'static
    {
        self.inner.lock().unwrap().on_complete(Box::new(f));
    }
}


pub struct Future<V: Send + 'static, E: Send + Clone + 'static> {
    value: Option<Result<V, E>>,
    next: Option<Box<FnMut(&Result<V, E>) -> () + Send>>
}

impl <V: Send + Clone, E: Send + Clone> Future<V , E> {

    /// Syntactic sugar for creating of AsyncPromise and extracting it's future
    pub fn asyncp<F>(f: F, executor: TSafe<Executor>) -> WrappedFuture<V, E>
        where F: FnMut() -> Result<V, E> + Send + 'static
    {
        let mut p: AsyncPromise<V, E> =
            AsyncPromise::new(Box::new(f), executor);
        p.future()
    }


    pub fn new() -> Future<V, E> {
        Future {
            value: None,
            next: None,
        }
    }

    /// Return current completion state
    pub fn is_completed(&self) -> bool {
        self.value.is_some()
    }

    /// Fills the value of the futures. This function is called from the promise of which the
    /// future belongs and it should never be called by the application code, otherwise it will
    /// lead to a breakdown of the future execution logic.
    pub fn complete(&mut self, result: Result<V, E>) {
        self.value = Some(result);

        if self.next.is_some() {
            if let Some(ref mut v) = self.value {
                if let Some(ref mut f) = self.next {
                    f(v);
                }
            }
        }
    }

    /// See mirror in WrappedFuture
    pub fn map<S>(&mut self, mut f: Box<FnMut(&V) -> Result<S, E> + Send>) -> WrappedFuture<S, E>
        where S: Send + Clone + 'static
    {
        let mut p: CompletablePromise<S, E> = CompletablePromise::new();
        let fut = p.future();
        self.next = Some(Box::new( move |v: &Result<V, E>| {
            if v.is_ok() {
                let x = v.as_ref().ok().unwrap();
                let result = f(x);
                p.complete(result);
            } else {
                let err = v.as_ref().err().unwrap().clone();
                let result: Result<S, E> = Err(err);
                p.complete(result);
            }
        }));

        if self.value.is_some() {
            if let Some(ref mut v) = self.value {
                if let Some(ref mut f) = self.next {
                    f(v)
                }
            }
        }

        fut
    }

    /// See mirror in WrappedFuture
    pub fn recover(&mut self, mut f: Box<FnMut(&E) -> Result<V, E> + Send>) -> WrappedFuture<V, E>  {
        let mut p: CompletablePromise<V, E> = CompletablePromise::new();
        let fut = p.future();
        self.next = Some(Box::new( move |v: &Result<V, E>| {
            if v.is_err() {
                let x = v.as_ref().err().unwrap();
                let result = f(x);
                p.complete(result);
            } else {
                let ok = v.as_ref().ok().unwrap().clone();
                let result: Result<V, E> = Ok(ok);
                p.complete(result);
            }
        }));

        if self.value.is_some() {
            if let Some(ref mut v) = self.value {
                if let Some(ref mut f) = self.next {
                    f(v)
                }
            }
        }

        fut
    }

    /// See mirror in WrappedFuture
    pub fn flat_map<S>(&mut self, mut f: Box<FnMut(&V) -> Result<WrappedFuture<S, E>, E> + Send>) -> WrappedFuture<S, E>
        where S: Send + Clone + 'static
    {
        let mut p: TSafe<CompletablePromise<S, E>> = tsafe!(CompletablePromise::new());
        let fut = p.lock().unwrap().future();
        self.next = Some(Box::new( move |v: &Result<V, E>| {
            if v.is_ok() {
                let x = v.as_ref().ok().unwrap();
                let result = f(x);
                if result.is_ok() {
                    let mut next_fut = result.ok().unwrap();
                    let p_clone = p.clone();
                    next_fut.on_complete(move |v| {
                        let r: Result<S, E> = if v.is_ok() {
                            Ok(v.as_ref().ok().unwrap().clone())
                        } else {
                            Err(v.as_ref().err().unwrap().clone())
                        };
                        p_clone.lock().unwrap().complete(r);
                    });
                } else {
                    let err = result.as_ref().err().unwrap().clone();
                    let result: Result<S, E> = Err(err);
                    p.lock().unwrap().complete(result);
                }
            } else {
                let err = v.as_ref().err().unwrap().clone();
                let result: Result<S, E> = Err(err);
                p.lock().unwrap().complete(result);
            }
        }));

        if self.value.is_some() {
            if let Some(ref mut v) = self.value {
                if let Some(ref mut f) = self.next {
                    f(v)
                }
            }
        }

        fut
    }

    /// See mirror in WrappedFuture
    pub fn on_complete(&mut self, mut f: Box<FnMut(&Result<V, E>) -> () + Send>)  {
        self.next = Some(Box::new( move |v: &Result<V, E>| { f(v) }));
        if self.value.is_some() {
            if let Some(ref mut v) = self.value {
                if let Some(ref mut f) = self.next {
                    f(v)
                }
            }
        }
    }
}

