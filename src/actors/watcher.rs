//! Event bus of watching
//!
//! This object includes to actor system and used for control events of mechanic of watching. This
//! is not public api. It's direct usage may damage internal logic of the system.

use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::actor_path::ActorPath;
use std::collections::HashMap;

pub enum WatchingEvents {
    Terminated
}

pub mod events {
    pub struct Terminated {}
}

pub struct Watcher {
    feed: HashMap<ActorPath, (ActorRef, Vec<ActorRef>)>,
    watchers: HashMap<ActorPath, u32>
}

impl Watcher {
    pub fn new() -> Watcher {
        Watcher {
            feed: HashMap::new(),
            watchers: HashMap::new()
        }
    }

    /// Subscribe the watcher actor to the events of the observed actor. All subscriptions will be
    /// automatically dropped after Terminated event will be registered.
    pub fn watch(&mut self, watcher: &ActorRef, mut observed: &ActorRef) {
        trace!("{} watch {}", watcher, observed);
        let obs_path = observed.path().clone();
        if self.feed.contains_key(&obs_path) {
            let (_, watchers) = self.feed.get_mut(&obs_path).unwrap();
            watchers.push((*watcher).clone());
        } else {
            self.feed.insert(obs_path, ((*observed).clone(), vec![(*watcher).clone()]));
        }

        let wt_path = watcher.path();
        if self.watchers.contains_key(&wt_path) {
            let counter = self.watchers.get_mut(&wt_path).unwrap();
            *counter = *counter + 1;
        } else {
            self.watchers.insert(wt_path, 1);
        }
    }

    /// Subscribe the watcher actor from the events of the observed actor
    pub fn unwatch(&mut self, mut watcher: &ActorRef, mut observed: &ActorRef) {
        trace!("{} unwatch {}", watcher, observed);
        let obs_path = observed.path().clone();
        if self.feed.contains_key(&obs_path) {

            let (_, watchers) = self.feed.get_mut(&obs_path).unwrap();

            let mut new: Vec<ActorRef> = Vec::new();
            let wat_path = watcher.path();

            for w in watchers.iter() {
                if w.path() != wat_path {
                    new.push((*w).clone());
                }
            }

            if new.len() > 0 {
                *watchers = new;
            } else {
                self.feed.remove(&obs_path);
            }
        }

        let wt_path = watcher.path();
        if self.watchers.contains_key(&wt_path) {
            let counter = self.watchers.get_mut(&wt_path).unwrap();
            *counter = *counter  - 1;
            if *counter <= 0 {
                self.watchers.remove(&wt_path);
            }
        }
    }

    /// Registers some event from an actor. This operation cause to send corresponding message to
    /// the all actors which was subscribed for source actor.
    pub fn register_event(&mut self, mut from: &ActorRef, event: WatchingEvents) {
        let obs_path = from.path().clone();
        match event {
            WatchingEvents::Terminated => {
                trace!("Registered event 'Terminated' from {}", from);
                if self.feed.contains_key(&obs_path) {
                    let (observed, watchers) =
                        self.feed.get_mut(&obs_path).unwrap();

                    for aref in watchers {
                        trace!("Send Terminated event message from {} to {}", from, aref);
                        aref.tell(Box::new(events::Terminated {}), Some(&observed))
                    }

                    self.feed.remove(&obs_path);
                }

                let wt_path = from.path();
                if self.watchers.contains_key(&wt_path) {
                    warn!("At the time of termination actor {} is subscribed for some events. You must prefer controlled unwatch!", from);

                    let mut fnv = Vec::new();
                    for (_, (obs, _)) in self.feed.iter_mut() {
                        fnv.push(obs.clone());
                    }

                    for obs in fnv {
                        self.unwatch(from, &obs);
                    }
                }
            }
        }
    }
}