use crate::actors::prelude::*;
use super::first;
use super::second;
use std::thread;
pub use match_downcast::*;

pub use std::sync::{Arc, Mutex};

pub fn run() {
    let mut system = LocalActorSystem::new();

    let mut second = system.lock().unwrap()
        .actor_of(second::props(), Some("second"));

    let mut first = system.lock().unwrap()
        .actor_of(first::props(&second), Some("first"));

    // To GetFlatResponse actor responds with his own message
    first.ask(&mut (*system.lock().unwrap()), msg!(first::commands::GetFlatResponse {}))
        .on_complete(|v| {
           if v.is_ok() {
               println!("Got response");
           } else {
               println!("AskTimeout");
           }
        });


    // To GetCascadeResponse actor responds message which will be created on result of request to
    // the second actor.
    let msg = msg!(first::commands::GetCascadeResponse { data: 100 });

    first.ask(&mut (*system.lock().unwrap()), msg)
        .on_complete(|v| {
            if v.is_ok() {
                let resp = v.as_ref().ok().unwrap();
                let resp = resp.get();

                match_downcast_ref!(resp, {
                    m: first::responses::CascadeResponse => {
                        println!("Got cascade response = {}", &m.data);
                    },
                    _ => ()
                });

            } else {
                println!("AskTimeout");
            }
        });

    thread::park();
}