use crate::actors::prelude::*;
use super::requestor::Requestor;
use super::msg_serializer::MsgSerializer;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

pub fn run() {

    // Create message serializer
    let msg_serializer = MsgSerializer::new();

    // Create local actor system
    let mut local_system = LocalActorSystem::new();

    // Create remote actor system
    let mut remote_system = RemoteActorSystem::new("127.0.0.1:5000".parse().unwrap(),
                                                   tsafe!(local_system.clone()),
                                                   tsafe!(msg_serializer));

    // Wait for some time, while network driver connects to the remote system
    thread::sleep(Duration::from_secs(1));

    // Select remote actor reference
    let mut selection = remote_system.actor_select("/root/storage");
    let storage = &mut selection[0];

    // Create local actor which will interact with remote system
    local_system.actor_of(Requestor::props(storage.clone()), Some("requestor"));

    thread::park();
}