use crate::actors::actor_ref_factory::{ActorRefFactory, ActorSelectError};
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::props::Props;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::remoting::remote_actor_ref::RemoteActorRef;
use crate::actors::remoting::delivery_error::{DeliveryError, DeliveryErrorReason};
use crate::actors::remoting::larid::*;
use crate::actors::message::Message;
use crate::actors::actor::Actor;
use crate::actors::remoting::net_controller::NetController;
use crate::actors::scheduler::Scheduler;
use crate::actors::dispatcher::Dispatcher;
use crate::actors::watcher::WatchingEvents;
use crate::actors::remoting::connection::{ConnectionData, ClientConnection};
use crate::actors::remoting::packet::{Packet, Opm};
use crate::actors::actor_path::ActorPath;
use crate::executors::executor::Executor;
use crate::common::tsafe::TSafe;
use crate::futures::future::WrappedFuture;
use crate::actors::remoting::messages_serializer::MessagesSerializer;
use bytes::{BytesMut, Bytes, Buf, BufMut, BigEndian};
use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::net::{TcpStream, SocketAddr};
use std::thread;
use std::time::Duration;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};


pub struct RemoteActorSystem {
    controller: RemoteNetController
}

impl RemoteActorSystem {
    pub fn new(addr: SocketAddr, host_system: TSafe<ActorRefFactory + Send>, messages_serializer: TSafe<MessagesSerializer + Send>) -> RemoteActorSystem {
        RemoteActorSystem {
            controller: RemoteNetController::new(addr, messages_serializer, host_system)
        }
    }
}

impl ActorRefFactory for RemoteActorSystem {

    fn actor_of(self: &mut Self, props: Props, name: Option<&str>) -> ActorRef {
        unimplemented!()
    }

    fn actor_select(&mut self, path: &str) -> Vec<ActorRef> {
        self.controller.actor_select(path)
    }

    fn stop(self: &mut Self, aref: &mut ActorRef) {
        unimplemented!()
    }

    fn dead_letters(self: &mut Self) -> ActorRef {
        unimplemented!()
    }

    fn watch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        unimplemented!()
    }

    fn unwatch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        unimplemented!()
    }
}

impl AbstractActorSystem for RemoteActorSystem {

    fn get_scheduler(&self) -> TSafe<Scheduler> {
        unimplemented!()
    }

    fn register_watch_event(&self, from: &ActorRef, event: WatchingEvents) {
        unimplemented!()
    }

    fn terminate(&mut self) {
        self.controller.stop()
    }

    fn add_dispatcher(&mut self, name: &str, dispatcher: TSafe<Dispatcher + Send>) {
        unimplemented!()
    }

    fn get_dispatcher(&self, name: &str) -> TSafe<Dispatcher + Send> {
        unimplemented!()
    }

    fn get_dispatchers(&self) -> TSafe<HashMap<String, TSafe<Dispatcher + Send>>> {
        unimplemented!()
    }

    fn get_executor(&self, name: &str) -> TSafe<Executor + Send> {
        unimplemented!()
    }

    fn get_nid(&mut self) -> String {
        unimplemented!()
    }
}

pub struct RequestError {

}

// ============================================================================

enum Request {
    ActorSelect(Sender<Result<Vec<(u32, String)>, RequestError>>)
}


#[derive(Clone)]
pub struct RemoteNetController {
    connection: TSafe<ClientConnection>,
    requests: TSafe<HashMap<u32, Request>>,
    id_counter: TSafe<u32>,
    boxed_self: Option<TSafe<Self>>,

    /// Larid map with actor path key
    larid_path_map: TSafe<HashMap<String, Larid>>,

    /// Larid map with id key
    larid_id_map: TSafe<HashMap<u32, Larid>>,

    /// Serializer for messages which transmitted through network
    messages_serializer: TSafe<MessagesSerializer + Send>,

    /// Deatch with of larid tables
    larid_watcher: TSafe<ActorRef>,

    /// Host actor system
    host_system: TSafe<ActorRefFactory + Send>
}

impl RemoteNetController {
    pub fn new(addr: SocketAddr, messages_serializer: TSafe<MessagesSerializer + Send>, host_system: TSafe<ActorRefFactory + Send>) -> RemoteNetController {
        let (s, r) = mpsc::channel();
        let requests = tsafe!(HashMap::new());
        let larid_path_map = tsafe!(HashMap::new());
        let larid_id_map = tsafe!(HashMap::new());
        let larid_watcher = tsafe!(host_system.lock().unwrap().actor_of(LaridWatcher::props(larid_path_map.clone(), larid_id_map.clone()), Some("larid_watcher")));


        let mut obj = RemoteNetController {
            connection: tsafe!(ClientConnection::new(addr, s)),
            requests: requests.clone(),
            id_counter: tsafe!(1),
            boxed_self: None,
            larid_id_map: larid_id_map.clone(),
            larid_path_map: larid_path_map.clone(),
            messages_serializer: messages_serializer.clone(),
            larid_watcher,
            host_system
        };

        let boxed_self = tsafe!(obj.clone());
        obj.boxed_self = Some(boxed_self.clone());

        Self::serve(requests, r, messages_serializer, boxed_self, larid_path_map, larid_id_map);


        obj
    }

    fn serve(requests: TSafe<HashMap<u32, Request>>,
             receiver: Receiver<ConnectionData>,
             messages_serializer: TSafe<MessagesSerializer + Send>,
             boxed_self: TSafe<Self>,
             larid_path_map: TSafe<HashMap<String, Larid>>,
             larid_id_map: TSafe<HashMap<u32, Larid>>) {
        thread::spawn(move || {
            loop {
                let cd = receiver.recv().unwrap();

                match cd {
                    ConnectionData::ReceivedPacket(packet) => {
                        match packet.opm {
                            Opm::ActorSelectResponse => {
                                let mut requests = requests.lock().unwrap();
                                let request = requests.remove(&packet.id);
                                if request.is_some() {
                                    match request.unwrap() {
                                        Request::ActorSelect(sender) => {
                                            let mut results = Vec::new();
                                            let refs = packet.body_as_actor_of_response();

                                            for r in refs {
                                                results.push((r, String::from("???")));
                                            }
                                            sender.send(Ok(results));
                                        }
                                        _ => ()
                                    }
                                }
                            },
                            Opm::SendMsg => {
                                let (marker, larid, rarid, blob) = packet.body_as_send_msg();
                                let msg = messages_serializer.lock().unwrap().from_binary(marker, blob);
                                if msg.is_err() {
                                    let err = msg.err().unwrap();
                                    error!("Unable to receive message from remote system doe to deserialization error {:?}", err);
                                    // Reply with delivery error
                                    // ...................................
                                    continue;
                                }
                                let msg = msg.unwrap();

                                let mut larid_id_map = larid_id_map.lock().unwrap();
                                let cu_larid = larid_id_map.get(&larid);
                                if cu_larid.is_none() {
                                    error!("Unable to receive message from remote system doe to target actor is died");
                                    // Reply with delivery error
                                    // ...................................
                                    continue;
                                }
                                let path = ActorPath::new(&format!("remote://{}", &rarid), None);
                                let sender: ActorRef = Box::new(RemoteActorRef::new(0, rarid, tsafe!(path), boxed_self.clone()));
                                let mut aref = (*cu_larid.as_ref().unwrap()).aref.clone();
                                aref.tell(msg, Some(&sender));
                            },
                            e => error!("Handled remote packet with unsupported opm '{:?}'", e)
                        }
                    },
                    ConnectionData::Closed => {
                        break;
                    }
                }
            }

            trace!("Remote processor stopped");
        });
    }

    pub fn actor_select(&mut self, path: &str) -> Vec<ActorRef> {
        let id = self.get_id();
        let (s,r) = mpsc::channel();
        self.requests.lock().unwrap().insert(id, Request::ActorSelect(s));


        loop {
            let packet = Packet::new_actor_of(id, path);

            if self.connection.lock().unwrap().send(packet) {
                break;
            } else {
                thread::sleep(Duration::from_secs(1));
            }
        }

        let selection = r.recv().unwrap();
        let selection = selection.ok().unwrap();

        let mut vec: Vec<ActorRef> = Vec::new();

        for (rarid, path) in selection {
            let path = ActorPath::new(&path, None);
            let aref = Box::new(RemoteActorRef::new(0, rarid, tsafe!(path), self.boxed_self.as_ref().unwrap().clone()));
            vec.push(aref);
        }


        //TODO Обработка тайматуров


        vec
    }

    pub fn get_id(&mut self) -> u32 {
        let mut id_counter = self.id_counter.lock().unwrap();
        *id_counter = *id_counter + 1;
        *id_counter
    }

    pub fn stop(&mut self) {
        self.connection.lock().unwrap().close();
        self.boxed_self = None;
        self.larid_path_map.lock().unwrap().clear();
        self.larid_id_map.lock().unwrap().clear();
        self.host_system.lock().unwrap().stop(&mut self.larid_watcher.lock().unwrap().clone())
    }
}

impl NetController for RemoteNetController {
    fn send_msg(&mut self, msg: Message, rcid: u32, rarid: u32, sender: Option<ActorRef>, far: ActorRef) {

        // Serialize message
        let bin = self.messages_serializer.lock().unwrap().to_binary(msg);
        if bin.is_err() {
            let err = bin.err().unwrap();
            error!("Unable to send message to remote system due to serialization error '{:?}'", err);
            if sender.is_some() {
                let mut sender = (*sender.as_ref().unwrap()).clone();
                sender.tell(msg!(DeliveryError { reason: DeliveryErrorReason::SerializationError}), Some(&far));
            }

            return;
        }
        let bin = bin.unwrap();

        // Obtain larid
        let larid = {
            if sender.is_some() {
                let path = sender.as_ref().unwrap().path().to_string();
                let id = self.get_id();
                let mut larid_path_map = self.larid_path_map.lock().unwrap();
                let larid = larid_path_map.get(&path);
                if larid.is_some() {
                    larid.as_ref().unwrap().id
                } else {
                    let sender = (*sender.as_ref().unwrap()).clone();
                    let larid_watcher = self.larid_watcher.lock().unwrap();
                    self.host_system.lock().unwrap().watch(&larid_watcher.clone(), &sender);

                    larid_path_map.insert(path, Larid { id, aref: sender.clone() });
                    let mut larid_id_map = self.larid_id_map.lock().unwrap();
                    larid_id_map.insert(id, Larid { id, aref: sender });
                    id
                }
            } else {
                0
            }
        };

        // Send packet to the connection
        let id = self.get_id();
        let packet = Packet::new_send_msg(id, bin.marker, &bin.blob[..], rarid, larid);

        // Get connection
        let result = self.connection.lock().unwrap().send(packet);
        if !result {
            error!("Unable to send message to remote system due to network error");
            if sender.is_some() {
                let mut sender = (*sender.as_ref().unwrap()).clone();
                sender.tell(msg!(DeliveryError { reason: DeliveryErrorReason::NetworkError}), Some(&far));
            }
        }
    }
}

impl Drop for RemoteActorSystem {
    fn drop(&mut self) {
        println!("RemoteActorSystem dropped")
    }
}


impl Drop for RemoteNetController {
    fn drop(&mut self) {
        println!("RemoteNetController dropped")
    }
}