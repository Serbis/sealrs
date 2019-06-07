//! Actor system is a container for actors runtime. She is used for actor creation,  stop and other
//! operations

use crate::common::tsafe::TSafe;
use crate::actors::props::Props;
use crate::actors::actor_path::ActorPath;
use crate::actors::actor_cell::ActorCell;
use crate::actors::actor::PoisonPill;
use crate::actors::default_dispatcher::DefaultDispatcher;
use crate::actors::dead_letters::DeadLetters;
use crate::actors::synthetic_actor::SyntheticActor;
use crate::actors::unbound_mailbox::UnboundMailbox;
use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::local_actor_ref::LocalActorRef;
use crate::actors::abstract_actor_ref::ActorRef;
use std::sync::{Arc, Mutex};


pub struct LocalActorSystem {
    /// Actors ids counter. His is used for set up actor name, if it was not be explicitly specified.
    nids: usize,

    /// Default dispatcher. Used if other dispatcher does not explicitly specified.
    pub dispatcher: TSafe<DefaultDispatcher>,

    /// Dead letter actor reference. Sending message through this reference has is very low cost,
    /// because message after drop to the mailbox, is simply destroyed without hes subsequent
    /// execution planning. */
    dead_letters: Option<ActorRef>
}

impl LocalActorSystem {
    /// Create new actor system protected by TSafe guard.
    pub fn new() -> TSafe<LocalActorSystem> {
        let cpu_count = num_cpus::get();
        let dispatcher = tsafe!(DefaultDispatcher::new(cpu_count as u32));
        let mut system = LocalActorSystem {
            nids: 0,
            dispatcher: dispatcher.clone(),
            dead_letters: None
        };

        let system = tsafe!(system);

        let dlp = tsafe!(ActorPath::new("deadLetters"));
        let dlm = DeadLetters::new();
        let dlc = ActorCell::new(system.clone(), dlp.clone(),tsafe!(SyntheticActor {}), 0, dispatcher.clone(), tsafe!(dlm));

        let boxed_dlc = tsafe!(dlc);


        system.lock().unwrap().dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp)));
        boxed_dlc.lock().unwrap().start(boxed_dlc.clone());

        system
    }
}

//TODO у всех ActorSystem убрать метод run

impl ActorRefFactory for LocalActorSystem {



    /// Creates the new actor from specified Props object and with specified name. If name does not
    /// explicitly specified, it will be generate automatically.
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    ///
    fn actor_of(self: &mut Self, props: Props, name: Option<&str>) -> ActorRef {
        let mailbox = tsafe!(UnboundMailbox::new());

        let mut aname: String;

        if name.is_some() {
            aname = name.unwrap().to_string();
        } else {
            aname = self.nids.to_string();
            self.nids = self.nids + 1;
        }

        let path = tsafe!(ActorPath::new(&aname));

        let mut cell = ActorCell::new(
            tsafe!(self.clone()),
            path.clone(),
            props.actor,
            0,
            self.dispatcher.clone(),
            mailbox
        );
        let boxed_cell = tsafe!(cell);

        boxed_cell.lock().unwrap().start(boxed_cell.clone());

        Box::new(LocalActorRef::new(boxed_cell, path))
    }

    /// Stop specified actor by it's reference. Suspends actor, cancels all timers, cleans mailbox
    /// and sends to it the PoisonPill message, which will be processed right away after the current
    /// message (if this call will made from actor's message handler) or depending on the stopped
    /// actor state (if it idle it will be stopped immediately, if not, after processing his current
    /// message )
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn stop(self: &mut Self, aref: &mut ActorRef) {
        // Attention, identical code exists in the PoisonPill handler
        let aref_cpy0 = aref.clone();
        let aref_cpy1 = aref.clone();
        let x = aref.cell();
        let mut cell = x.lock().unwrap();
        cell.suspend();
        // +++ cell.actor.timers().cancelAll();
        cell.mailbox.lock().unwrap().clean_up(aref_cpy0, self.dead_letters());
        cell.force_send(aref.cell().clone(), Box::new(PoisonPill {}), None, aref_cpy1);
    }

    /// Return deadLetter actor reference
    fn dead_letters(self: &mut Self) -> ActorRef {
        match &self.dead_letters {
            Some(d) =>  {
                (*d).clone()
            }
            _ => panic!("Dead letter is empty")
        }
    }
}

impl AbstractActorSystem for LocalActorSystem {

}

impl Clone for LocalActorSystem {
    fn clone(&self) -> Self {

        let dead_letter: Option<ActorRef> = match &self.dead_letters {
            Some(v) => Some((*v).clone()),
            None => None
        };
        LocalActorSystem {
            nids: self.nids,
            dispatcher: self.dispatcher.clone(),
            dead_letters: dead_letter //self.dead_letters.clone()
        }
    }
}