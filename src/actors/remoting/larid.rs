//! Local actor reference identifier and his watcher
//!
//! This object is the representation of a local actor reference for a remote connection.
//! He is used as a binder between RemoteActorRef in a remote system and LocalActorRef in a local system.

use crate::actors::prelude::*;
use crate::actors::watcher::events::Terminated;
use crate::common::tsafe::TSafe;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Larid {
    pub id: u32,
    pub aref: ActorRef
}

pub struct LaridWatcher {
    larid_path_map: TSafe<HashMap<String, Larid>>,
    larid_id_map: TSafe<HashMap<u32, Larid>>,
}

impl LaridWatcher {
    pub fn props(larid_path_map: TSafe<HashMap<String, Larid>>,
                 larid_id_map: TSafe<HashMap<u32, Larid>>) -> Props {
        Props::new(tsafe!(LaridWatcher::new(larid_path_map, larid_id_map)))
    }

    pub fn new(larid_path_map: TSafe<HashMap<String, Larid>>,
               larid_id_map: TSafe<HashMap<u32, Larid>>) -> LaridWatcher {
        LaridWatcher {
            larid_path_map,
            larid_id_map
        }
    }
}

impl Actor for LaridWatcher {
    fn receive(&mut self, msg: Message, mut ctx: ActorContext) -> HandleResult {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: Terminated => {
                let path = ctx.sender.path().to_string();
                let larid = self.larid_path_map.lock().unwrap().remove(&path);
                if larid.is_some() {
                    let id = larid.as_ref().unwrap().id;
                    self.larid_id_map.lock().unwrap().remove(&id);
                    trace!("Removed larid by LaridWatcher [ id={}, path={} ]", id, path);
                }
            },
            _ => return Ok(false)
        });

        Ok(true)
    }
}