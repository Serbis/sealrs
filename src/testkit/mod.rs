//! A set of tools for testing code based on the library
//!
//! Testing asynchronous code is complicated. In most cases, it's comes down to few high-level
//! integration tests, which cover only main functions of the program. And if with integrations
//! tests all looks more or less good, then with unit testing is all is no. Practically is
//! impossible perform quality unit testing, if framework developer does was not presented explicit
//! tools for this. This modules presents this tools and allows to library users, perform deep unit
//! testing of their code. For more details, see docs for concrete submodule.

#[macro_use]
pub mod macrodef;
#[macro_use]
pub mod actors;