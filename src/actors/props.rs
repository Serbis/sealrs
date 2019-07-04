//!Constructor of user defined actor
//!
//! This structure is used, for indicates to ActorSystem, how to create an actor

use crate::actors::actor::Actor;
use crate::actors::supervision::SupervisionStrategy;
use crate::common::tsafe::TSafe;

pub struct Props {

    /// User defined actor instance
    pub actor: TSafe<Actor + Send>,

    /// Name of dispatcher on which actor must work
    pub dispatcher: String,

    /// Supervision strategy for the actor instance
    pub supervision_strategy: SupervisionStrategy
}

impl Props {
    pub fn new(actor: TSafe<Actor + Send>) -> Props {
        Props {
            actor,
            dispatcher: String::from("default"),
            supervision_strategy: SupervisionStrategy::Restart
        }
    }

    /// Sets dispatcher name on which the actor must work. By default exists two type of
    /// dispatchers: 'default' and 'pinned'. If you want to use other dispatchers types, you need
    /// register it's in the actor system.
    pub fn with_dispatcher(mut self, name: &str) -> Props {
        self.dispatcher = String::from(name);
        self
    }

    /// Sets supervision strategy for the actor instance
    pub fn with_supervision_strategy(mut self, strategy: SupervisionStrategy) -> Props {
        self.supervision_strategy = strategy;
        self
    }
}