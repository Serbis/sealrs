use sealrs::futures::future::WrappedFuture;
use futures::future;
use futures::{Async, Poll};

pub struct SealFuture<V: Send  + Clone + 'static, E: Send  + Clone + 'static> {
    fut: WrappedFuture<V, E>
}

impl <V: Send  + Clone + 'static, E: Send  + Clone + 'static> SealFuture<V, E> {
    pub fn new(fut: WrappedFuture<V, E>) -> SealFuture<V, E> {
        SealFuture {
            fut
        }
    }
}

impl <V: Send + Clone, E: Send + Clone> future::Future for SealFuture<V, E> {
    type Item = V;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if !self.fut.is_completed() {
            Ok(Async::NotReady)
        } else {
            let inner = self.fut.inner.lock().unwrap();
            let value = inner.value.as_ref().unwrap();
            if value.is_ok() {
                let ok_result = value.as_ref().ok().unwrap().clone();
                Ok(Async::Ready(ok_result))
            } else {
                let err_result = value.as_ref().err().unwrap().clone();
                Err(err_result)
            }

        }
    }
}