//! Actor path representation
//!
//! This object is used for many needs - comparing references, printing log messages, actor position
//! checks and for many other things.
//!
use std::fmt;

pub struct ActorPath {

    /// Actor name - which user set, when create an actor through actor_of function. If it does not
    /// was specified, this field will contain automatically generated name.
    pub name: String
}

impl ActorPath {
    pub fn new(name: &str) -> ActorPath {
        ActorPath {
            name: name.to_string()
        }
    }
}

impl Clone for ActorPath {
    fn clone(&self) -> Self {
        ActorPath {
            name: self.name.clone()
        }
    }
}

impl fmt::Display for ActorPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "/{}", self.name)
    }
}

impl PartialEq for ActorPath {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ActorPath {}