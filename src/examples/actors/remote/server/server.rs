use crate::actors::prelude::*;
use super::storage::Storage;
use super::msg_serializer::MsgSerializer;
use std::thread;
use std::sync::{Arc, Mutex};

pub fn run() {

    // Create message serializer
    let msg_serializer = MsgSerializer::new();

    // Create network actor system
    let mut net_system = NetworkActorSystem::new("127.0.0.1:5000".parse().unwrap(),
                                                 tsafe!(msg_serializer));
    // Create work actor
    net_system.actor_of(Storage::props(), Some("storage"));

    thread::park()
}