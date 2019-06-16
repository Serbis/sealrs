//! Classic realization of Future / Promise paradigm
//!
//! # Table of Contents
//! 1. [Introduction](#introduction)
//! 2. [CompletablePromise](#completablepromise)
//! 3. [AsyncPromise](#asyncpromise)
//! 4. [Failures control](#failures-control)
//! 5. [Future combinators](#future-combinators)
//!     1. [map](#map)
//!     2. [flat_map](#flat_map)
//!     3. [recover](#recover)
//!     4. [on_complete](#on_complete)
//!     5. [Example of chaining](#example-of-chaining)
//! 6. [Requirements to V and E](#requirements-to-v-and-e)
//! 7. [Blocking operations](#blocking-operations)
//! 8. [Futures conversion](#futures-conversion)
//!
//! # Introduction
//!
//! Future is a container for some value which will be received after a certain time but not right
//! after creation. Promise is a function, which produce a value and pass it to a Future. Right
//! after Future receives a value, she calls callback functions, attached by the user, which may
//! transform the value and do some actions with finite result. How Promise calculate a value is
//! not defined. Besides on this, Promise presented as abstract trait Promise<V, E>, which implement
//! specific realizations. Type V presents some success result and E presents an error. In this
//! library exists few predefined realizations of Promise.
//!
//! # CompletablePromise
//!
//! This promise may be completed manually from the user side. Let's see on example.
//!
//! ```
//! let mut p: CompletablePromise<u32, TSafe<Fail + Send>> = CompletablePromise::new();
//! let mut fut = p.future();
//!
//! fut.on_complete(|v| {
//!     println!("Result={}", v.as_ref().ok().unwrap());
//! });
//!
//! thread::spawn(move || {
//!     p.success(100);
//! });
//! ```
//!
//! At first line, new Promise is created. You must explicitly specify him result and error types,
//! because for now, compiler does not may infer it automatically. Next, we extract the Future
//! stored in the Promise. This is exactly that Future which will completes the that promise.
//! At line forth, we attach on_complete callback to the Future. We learn about it later in this
//! doc. At the moment, you must know, that on_complite is called on finite result of the future's
//! chain and accept Result value, which may be as is 'Ok' as is 'Err'. This value represents result
//! of all commutation. For now, in the example code, error value is not produced, and we may simply
//! unwrap 'Ok' result. Next we create new thread and move Promise into it. Further in this another
//! thread, calls success method of Promise. This call, lead to that passed to it value, will be
//! filled to the future, and right after than, on_complete callback will be called. You see
//! 'Result=100' on the your screen.
//!
//! This is not very useful code and it is written only for demonstration, how Promise / Future is
//! works together. In the real world, you may want to perform some  more very complicated actions
//! in the on_complete callback. For example you may perform an http request to a remote server,
//! based on the computation results.
//!
//! # AsyncPromise
//!
//! This promise type accept a closure and executor. Closure will be planned on the specified
//! executor and when it completes, promise will be automatically completes with result of its
//! execution. See to example.
//!
//! ```
//! let task = Box::new(|| Ok(50 + 50));
//!
//! let mut p: AsyncPromise<u32, TSafe<Fail + Send>> =
//!     AsyncPromise::new(task, executor);
//!
//! let mut fut = p.future();
//!
//! fut.on_complete(|v| {
//!     println!("Result={}", v.as_ref().ok().unwrap());
//! });
//! ```
//!
//! First we create the task. Then as in the previous example we create new promise, but now it is
//! AsyncPromise. To the constructor we put task and executor. Right after that, task will be
//! planned to execution on the executor. Next as is earlier, is set on_complete callback. But for
//! now, we don't need to complete the promise manually. This will happen automatically after the
//! task will be completed.
//!
//! This type of promise is most of more usable for various needs. Therefore for him exists special
//! syntactic sugar, which prevents creation of Promise by the user and simply returns prepared and
//! extracted future.
//!
//! ```
//! let mut fut: WrappedFuture<u32, TSafe<Fail + Send>> =
//!     Future::asyncp(task, executor)
//! ```
//!
//! # Failures control
//!
//! Before we talk about futures combinators, need to say about its failures control mechanism. This
//! realization of Future / Promise suggests that at any stage of execution, may occurs some error.
//! It is not panic, but application level error, when programmers consciously creates error
//! precedent. For this, used Result type, which represents the two possible situations - success or
//! failure. Each piece of code executed in the promise / future context must always explicitly
//! specify how it completed - success or with some error. On this is based logic of some
//! combinators, which may be executed only for success or for failed result. Upper type of error
//! specified as E type at promise creation stage.
//!
//! # Future combinators
//!
//! Combinators is a functions which may be attached to the future. They will be called right after
//! future was filled and may transform result with some algorithms. All combinators except of
//! on_complete produce a new future, value of then depends on result of calling attached combinator
//! on parent future. This construction is called - chain of futures. In all the examples, work with
//! the following “future” will be assumed:
//!
//! ```
//! let mut fut: WrappedFuture<u32, TSafe<Fail + Send>> =
//!     Future::asyncp(|| Ok(500 + 500), executor);
//! ```
//!
//! ## map
//!
//! Transforms result of the current future to result of some another type. Never calls if previous
//! result was error.
//!
//! ```
//! fut.map(|v| {
//!       Ok(format!("{}", v))
//! })
//! ```
//!
//! In this example, value of u32 type will be transformed to the result of String type.
//!
//! ## flat_map
//!
//! Transforms result of the current future to another future. Never calls if previous result was
//! error.
//!
//! ```
//! fut.flat_map(move |v| {
//!     let vc = *v;
//!     let fut_inner: WrappedFuture<String, TSafe<Fail + Send>> =
//!     Future::asyncp(move || {
//!         let r = format!("{}", vc + 1000);
//!         Ok(r)
//!     }, executor1.clone());
//!     Ok(fut_inner)
//! });
//! ```
//!
//! In this example we create new future in the flat_map and return in as result. How it work is
//! difficult to understand without understanding of chaining (read about it in the special
//! paragraph). All next combinators which will ba attached after flat_map call, will be attached to
//! the inner future which was returned as result from the flat_map. For example, if you attach map
//! as next combinator, it will accept String type as result with value calculated in the inner
//! future.
//!
//! ## recover
//!
//! Recovers from error if it's occurs at previous stage. Never calls if previous result was success.
//!
//! ```
//! fut.map(|v| {
//!     let err = tsafe!(MyError::ExampleError { text: String::from("Oops!") });
//!     Err(err)
//! }).recover(|e| {
//!     Ok(100)
//! });
//! ```
//! In upper map was produced an error. Recovery combinator handle her and returns some default value
//! as recovery result. Should be noted, than self combinator may produce error. This error will be
//! lifted to a subsequent combinator.
//!
//! ## on_complete
//!
//! Final combinator, because it does not return new future and as consequence of this, you can't
//! unable to extend chain after it.
//!
//! ```
//! fut.on_complete(|v| {
//!     match v {
//!         Ok(result) => println!("Result={}", result),
//!         Err(error) => println!("Error={}", error.lock().unwrap())
//!     }
//! });
//! ```
//!
//! Body of the combinator accepts a raw result of the last future. This may be both success and
//! error and you mast detect who is this.
//!
//! ## Example of chaining
//!
//! Example of how combinators may be chained to work together.
//!
//! ```
//! let vst = 1000;
//!
//! let mut fut: WrappedFuture<u32, TSafe<Fail + Send>> =
//!     Future::asyncp(move || {
//!         if vst > 500 {
//!             Ok(vst + 1000)
//!         } else {
//!             let err: TSafe<Fail + Send> =
//!             tsafe!(MyError::ExampleError { text: String::from("Vst must be > 500!") });
//!             Err(err)
//!         }
//! }, executor.clone());
//!
//! fut.map(|v| {
//!     Ok(format!("{}", *v))
//! }).recover(|e| {
//!     println!("Recovered to default 100 because error occurs -> {}", e.lock().unwrap());
//!     Ok(String::from("100"))
//! }).on_complete(|v| {
//!     println!("Result={}", v.as_ref().unwrap())
//! });
//! ```
//!
//! In this example, depends on value of the vst variable, chain will be executed in two ways. If vst
//! is gross than 500, it will be converted to string and printed at finish. If less, it will
//! case error, which will cause to drop map combinator and call recover. In him we print the message
//! about occurred error, and return default converted to string value 100, which will also be
//! printed at finish.
//!
//! # Requirements to V and E
//!
//! To the types T and E exists some requirements:
//! * Types must not conflict with marker trait Send (Another words, they must be sized)
//! * Types must implements trait Clone
//!
//! Obviously that not all types supports this requirements. Simplest way for get around this
//! restrictions is use interior mutability. If your object is conflicts with Promise type's
//! requirements, simply wrap it to Arc\<Mutex\<T>> and all will be work.
//!
//! # Blocking operations
//!
//! Future have feature for work in fully synchronous way, when the calling thread blocks on waiting
//! on that the future will be completed. Immediately want to warn that this option allowed only in
//! test scenarios and when you need to interact with future from a synchronous code. In all other
//! situations it is a very scary antipattern and his usage may cause to very serious errors in the
//! asynchronous logic. WrappedFuture object presents two synchronous methods.
//!
//! * Ready - waits while future will be completed in a specified timeout. If future was completed
//! before timeout was reached, true will be returned. If timeout occurs, returned false.
//!
//! ```
//! let mut fut1: WrappedFuture<u32, TSafe<Fail + Send>> =
//!     Future::asyncp(|| {
//!         thread::sleep(Duration::from_secs(1));
//!         Ok(100)
//!     }, executor.clone());
//!
//! let ready = fut1.ready(Duration::from_secs(3));
//!
//! match ready {
//!     true => {
//!         match fut1.get_value() {
//!             Ok(value) => {
//!                 println!("fut1 completed with Ok({})", value);
//!             },
//!             Err(error) => {
//!                 println!("fut1 completed with Err({})", error.lock().unwrap());
//!             }
//!         }
//!     },
//!     false => {
//!         println!("fut1 does not completed with timeout");
//!     }
//! }
//! ```
//!
//! * Result - waits while future will be completed in a specified timeout. If future was completed
//!  before timeout was reached, value of future packed in Ok will be returned. If timeout occurs, returned
//!  Err(TimeoutError).
//!
//! ```
//! let mut fut2: WrappedFuture<u32, TSafe<Fail + Send>> =
//!     Future::asyncp(|| {
//!         thread::sleep(Duration::from_secs(1));
//!         Ok(100)
//!     }, executor.clone());
//!
//! let result = fut2.result(Duration::from_secs(3));
//!
//! match result {
//!     Ok(value) => {
//!         match value {
//!             Ok(value) => {
//!                 println!("fut2 completed with Ok({})", value);
//!             },
//!             Err(error) => {
//!                 println!("fut2 completed with Err({})", error.lock().unwrap());
//!             }
//!         }
//!     },
//!     Err(_) => {
//!         println!("fut2 does not completed with timeout");
//!     }
//! }
//! ```
//!
//! Full code of the example you may find in the examples submodule.
//!
//!
//! # Futures conversion
//!
//! Despite the fact that the library futures internally is very different from default Rust futures, it may
//! be simply converted to they. This is possible because both futures type uses identical completion paradigm.
//! For convert sealrs futures to the rust futures, you need implement simple converter. I intentionally did not
//! turn it on to the library in order to don't creates the 'futures' crate versions incompatibility. You may
//! copy this file to anywhere in your project and use it.
//!
//! ```
//! use sealrs::futures::future::WrappedFuture;
//! use futures::future;
//! use futures::{Async, Poll};
//!
//! pub struct SealFuture<V: Send  + Clone + 'static, E: Send  + Clone + 'static> {
//!     fut: WrappedFuture<V, E>
//! }
//!
//! impl <V: Send  + Clone + 'static, E: Send  + Clone + 'static> SealFuture<V, E> {
//!     pub fn new(fut: WrappedFuture<V, E>) -> SealFuture<V, E> {
//!         SealFuture {
//!             fut
//!         }
//!     }
//! }
//!
//! impl <V: Send + Clone, E: Send + Clone> future::Future for SealFuture<V, E> {
//!     type Item = V;
//!     type Error = E;
//!
//!     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//!         if !self.fut.is_completed() {
//!             Ok(Async::NotReady)
//!         } else {
//!             let inner = self.fut.inner.lock().unwrap();
//!             let value = inner.value.as_ref().unwrap();
//!             if value.is_ok() {
//!                 let ok_result = value.as_ref().ok().unwrap().clone();
//!                 Ok(Async::Ready(ok_result))
//!             } else {
//!                 let err_result = value.as_ref().err().unwrap().clone();
//!                 Err(err_result)
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! As example of usage of this converter may be presented json-rpc server based on the 'jsonrpc-http-server' crate.
//! Here requests handler may work in asynchronous mode, in that it return a rust future with expected result
//! instead of a result itself. You may interconnect sealrs future with this server with the following way:
//!
//! ```
//! let mut io = IoHandler::new();
//!
//! io.add_method("say_hello", move |_: Params| {
//!     // Create sealrs future
//!     let fut = Future::asyncp(|| {
//!         Ok(Value::String(format!("hello", v)))
//!     }, (&executor).clone());
//!
//!     // Convert sealrs future to 'futures' future
//!     SealFuture::new(fut)
//! });
//!
//! let server = ServerBuilder::new(io)
//!     .start_http(&"127.0.0.1:3030".parse().unwrap())
//!     .expect("Unable to start RPC server");
//!
//! server.wait();
//! ```
//!
//! Need pay attention that future which will be passed to the converter must be unclosed, which means that the
//! future may contains any sorts of a combinators (map, recover, flat_map and etc.), but not on_complete, because
//! he never returns any result.

pub mod future;
pub mod promise;
pub mod completable_promise;
pub mod async_promise;