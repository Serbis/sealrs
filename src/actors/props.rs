//!Constructor of user defined actor
//!
//! This structure is used, for indicates to ActorSystem, how to create an actor

use crate::actors::actor::Actor;
use crate::common::tsafe::TSafe;

pub struct Props {

    /// User defined actor instance
    pub actor: TSafe<Actor + Send>
}

impl Props {
    pub fn new(actor: TSafe<Actor + Send>) -> Props {
        Props {
            actor
        }
    }
}