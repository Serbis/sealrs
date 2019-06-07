//! This library contains various useful abstractions used for construct of reactive and
//! non-blocking asynchronous apps with high cpu-time utilisation. He contains few modules. Each
//! module have it's own reach documentation. See concrete module for more info.

#[macro_use] extern crate failure;
#[macro_use] pub mod common;
#[macro_use] pub mod testkit;
pub mod actors;
pub mod executors;
//pub mod exps;
pub mod futures;
pub mod examples;



