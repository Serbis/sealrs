use std::any::Any;

pub type Message = Box<Any + Send>;