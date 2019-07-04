//! Actor path representation
//!
//! This object is used for many needs - comparing references, printing log messages, actor position
//! checks and for many other things.
//!
use crate::common::tsafe::TSafe;
use std::fmt;
use std::hash::{Hash, Hasher};

pub struct ActorPath {

    /// Actor name - which user set, when create an actor through actor_of function. If it does not
    /// was specified, this field will contain automatically generated name.
    pub name: String,

    /// Parent path segment
    pub parent: Option<TSafe<ActorPath>>
}

impl ActorPath {
    pub fn new(name: &str, parent: Option<TSafe<ActorPath>>) -> ActorPath {
        ActorPath {
            name: name.to_string(),
            parent
        }
    }
}

impl Clone for ActorPath {
    fn clone(&self) -> Self {
        ActorPath {
            name: self.name.clone(),
            parent: self.parent.clone()
        }
    }
}

impl fmt::Display for ActorPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.parent.is_some() {
            let parent = self.parent.as_ref().unwrap().lock().unwrap();
            let pstr = parent.to_string();
            write!(f, "{}/{}", pstr, self.name)
        } else {
            write!(f, "/{}", self.name)
        }
    }
}

impl PartialEq for ActorPath {
    fn eq(&self, other: &Self) -> bool {
        if self.parent.is_some() {
            let p1 = self.parent.as_ref().unwrap().lock().unwrap().to_string();
            let p2 = other.parent.as_ref().unwrap().lock().unwrap().to_string();
            let p_eq = p1 == p2;
            p_eq  == true && self.name == other.name
        } else {
            self.name == other.name
        }


    }
}

impl Eq for ActorPath {}

impl Hash for ActorPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}