//! Actor system which may accept connections from other systems.
//!
//! She is fully clone of the LocalActorSystem, exclude that she has special network controller,
//! allows cooperation between other systems though network.

use crate::actors::actor_ref_factory::{ActorRefFactory, ActorSelectError};
use crate::actors::abstract_actor_system::AbstractActorSystem;
use crate::actors::props::Props;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::scheduler::Scheduler;
use crate::actors::dispatcher::Dispatcher;
use crate::actors::watcher::WatchingEvents;
use crate::executors::executor::Executor;
use crate::actors::remoting::messages_serializer::MessagesSerializer;
use crate::actors::remoting::net_controller::NetController;
use crate::actors::remoting::remote_actor_ref::RemoteActorRef;
use crate::actors::remoting::delivery_error::{DeliveryError, DeliveryErrorReason};
use crate::actors::remoting::larid::*;
use crate::actors::message::Message;
use crate::common::tsafe::TSafe;
use crate::futures::future::WrappedFuture;
use crate::actors::remoting::connection::{ConnectionData, ServerConnection};
use crate::actors::remoting::packet::{Packet, Opm};
use crate::actors::remoting::id_counter::IdCounter;
use super::acceptor::Acceptor;
use crate::actors::actor_path::ActorPath;
use crate::actors::actor_cell::ActorCell;
use crate::actors::actor::PoisonPill;
use crate::actors::default_dispatcher::DefaultDispatcher;
use crate::actors::pinned_dispatcher::PinnedDispatcher;
use crate::actors::dead_letters::DeadLetters;
use crate::actors::synthetic_actor::SyntheticActor;
use crate::actors::unbound_mailbox::UnboundMailbox;
use crate::actors::local_actor_ref::LocalActorRef;
use crate::actors::watcher::Watcher;
use crate::actors::wrapped_dispatcher::WrappedDispatcher;
use crate::actors::supervision::SupervisionStrategy;
use crate::futures::future::Future;
use std::mem;
use std::collections::hash_map::HashMap;
use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex};
use std::net::{TcpStream, SocketAddr};
use std::thread;
use std::time::Duration;
use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub struct NetworkActorSystem {
    /// Actors ids counter. His is used for set up actor name, if it was not be explicitly specified.
    nids: TSafe<usize>,

    /// Default dispatcher. Used if other dispatcher does not explicitly specified.
    dispatchers: TSafe<HashMap<String, TSafe<Dispatcher + Send>>>,

    /// Dead letter actor reference. Sending message through this reference has is very low cost,
    /// because message after drop to the mailbox, is simply destroyed without hes subsequent
    /// execution planning. */
    dead_letters: Option<ActorRef>,

    /// Internal tasks scheduler
    scheduler: TSafe<Scheduler>,

    /// Watcher event bus
    watcher: TSafe<Watcher>,

    /// Root guardian synthetic actor
    root: Option<TSafe<ActorCell>>,

    /// Path of the root guardian actor
    root_path: TSafe<ActorPath>,

    /// Network controller
    controller: Option<TSafe<ServerNetController>>,
}

impl NetworkActorSystem {
    pub fn new(addr: SocketAddr, messages_serializer: TSafe<MessagesSerializer + Send>) -> NetworkActorSystem {
        let cpu_count = num_cpus::get();
        let def_dispatch = tsafe!(DefaultDispatcher::new(cpu_count as u32));
        let mut dispatchers: HashMap<String, TSafe<Dispatcher + Send>> = HashMap::new();
        dispatchers.insert(String::from("default"), def_dispatch.clone());

        let root_path = tsafe!(ActorPath::new("root", None));

        let mut system = NetworkActorSystem {
            nids: tsafe!(0),
            dispatchers: tsafe!(dispatchers),
            dead_letters: None,
            root: None,
            root_path: root_path.clone(),
            scheduler: tsafe!(Scheduler::new()),
            watcher: tsafe!(Watcher::new()),
            controller: None,
        };

        let system_safe = tsafe!(system.clone());


        let root = ActorCell::new(
            system_safe.clone(),
            root_path.clone(),
            tsafe!(SyntheticActor {}),
            0,
            def_dispatch.clone(),
            tsafe!(UnboundMailbox::new()),
            None,
            SupervisionStrategy::Resume
        );
        let root_safe = tsafe!(root);

        let dlp = tsafe!(ActorPath::new("deadLetters", Some(root_path.clone())));
        let dlm = DeadLetters::new();
        let mut dlc = ActorCell::new(
            system_safe.clone(),
            dlp.clone(),
            tsafe!(SyntheticActor {}),
            0,
            def_dispatch.clone(),
            tsafe!(dlm),
            Some(root_safe.clone()),
            SupervisionStrategy::Resume);

        dlc.stopped = false;

        let boxed_dlc = tsafe!(dlc);

        //system.lock().unwrap().dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp)));
        system_safe.lock().unwrap().dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp.clone())));
        system.dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp.clone())));
        system_safe.lock().unwrap().root = Some(root_safe.clone());
        system.root = Some(root_safe);

        let net_controller = tsafe!(ServerNetController::new(addr, system_safe.clone(), messages_serializer));

        system_safe.lock().unwrap().controller = Some(net_controller.clone());
        system.controller = Some(net_controller);
        boxed_dlc.lock().unwrap().start(boxed_dlc.clone());

        system
    }
}

impl ActorRefFactory for NetworkActorSystem {

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
            aname = self.get_nid();
        }

        let path = tsafe!(ActorPath::new(&aname, Some(self.root_path.clone())));

        let dispatcher: TSafe<Dispatcher + Send> = {
            match &(props.dispatcher)[..] {
                "default" => {
                    let d = self.dispatchers.lock().unwrap();
                    let d = d.get(&String::from("default"));
                    let d = d.as_ref().unwrap();
                    (*d).clone()
                },
                "pinned" => tsafe!(PinnedDispatcher::new()),
                other => {
                    let dispatchers = self.dispatchers.lock().unwrap();
                    let dispatcher = dispatchers.get(&String::from(other));

                    match dispatcher {
                        Some(d) => d.clone(),
                        None => panic!("Dispatcher with name '{}' does not registered", other)
                    }
                }
            }
        };


        let cell = ActorCell::new(
            tsafe!(self.clone()),
            path.clone(),
            props.actor,
            0,
            dispatcher,
            mailbox,
            self.root.clone(),
            props.supervision_strategy
        );
        let boxed_cell = tsafe!(cell);

        {
            let mut root = self.root.as_ref().unwrap().lock().unwrap();
            let exists = root.childs.get(&aname);
            if exists.is_some() {
                panic!("Unable to create actor with existed name '{}'", aname)
            }
            root.childs.insert(aname, boxed_cell.clone());
        }

        let f = boxed_cell.lock().unwrap().start(boxed_cell.clone());
        f();

        Box::new(LocalActorRef::new(boxed_cell, path))
    }

    fn actor_select(&mut self, path: &str) -> Vec<ActorRef> {
        let path = String::from(path);
        let mut segs: VecDeque<&str> = path.split("/").collect();

        let mut selection = Vec::new();
        let mut cur_cell = Some(self.root.as_ref().unwrap().clone());

        if segs.len() == 0 {
            return selection;
        }

        segs.pop_front().unwrap();
        segs.pop_front().unwrap();

        while segs.len() > 0 {
            let seg = segs.pop_front().unwrap();

            let cell = cur_cell.as_ref().unwrap().clone();
            let cell = cell.lock().unwrap();
            let t_cell = cell.childs.get(seg);

            if t_cell.is_some() {
                cur_cell = Some(t_cell.unwrap().clone());
            } else {
                cur_cell = None;
                break;
            }
        }

        if cur_cell.is_some() {
            let cell = cur_cell.unwrap();
            let path = cell.lock().unwrap().path.clone();
            let aref = LocalActorRef::new(cell, path);
            selection.push(Box::new(aref));
        }

        selection
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
        // Remove from root childs
        {
            let mut root = self.root.as_ref().unwrap().lock().unwrap();
            let aname = aref.path().name;
            let exists = root.childs.get(&aname);
            if exists.is_some() {
                root.childs.remove(&aname);
            }
        }

        // Attention, identical code exists in the PoisonPill handler
        let aref_cpy0 = aref.clone();
        let aref_cpy1 = aref.clone();
        let x = aref.cell();
        let mut cell = x.lock().unwrap();
        cell.suspend();
        cell.mailbox.lock().unwrap().clean_up(aref_cpy0, self.dead_letters());
        cell.force_send(aref.cell().clone(), msg!(PoisonPill {}), None, aref_cpy1);
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

    /// Register watcher for receive 'watching events' from observed actor
    fn watch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        self.watcher.lock().unwrap().watch(watcher, observed);
    }

    /// Unregister watcher from receive 'watching events' from observed actor
    fn unwatch(&mut self, watcher: &ActorRef, observed: &ActorRef) {
        self.watcher.lock().unwrap().unwatch(watcher, observed);
    }
}

impl AbstractActorSystem for NetworkActorSystem {
    /// Returns actor system scheduler
    fn get_scheduler(&self) -> TSafe<Scheduler> {
        self.scheduler.clone()
    }

    /// Register new watching event from the specified actor
    fn register_watch_event(&self, from: &ActorRef, event: WatchingEvents) {
        self.watcher.lock().unwrap().register_event(&from, event);
    }

    /// Stops the actor system
    fn terminate(&mut self) {
        let mut root = self.root.as_ref().unwrap().clone();
        root.lock().unwrap().stop(root.clone());
        let d_list = self.dispatchers.lock().unwrap();
        for (_, d) in d_list.iter() {
            d.lock().unwrap().stop();
        }
        self.dead_letters = None;

        let controller = mem::replace(&mut self.controller, None);
        controller.unwrap().lock().unwrap().stop();
    }

    /// Adds new dispatcher to the system. For now supporter only default dispatcher replacing
    fn add_dispatcher(&mut self, name: &str, dispatcher: TSafe<Dispatcher + Send>) {
        match name {
            "default" => {
                self.dispatchers.lock().unwrap().get("default").unwrap().lock().unwrap().stop();

                let root = ActorCell::new(
                    tsafe!(self.clone()),
                    self.root_path.clone(),
                    tsafe!(SyntheticActor {}),
                    0,
                    dispatcher.clone(),
                    tsafe!(UnboundMailbox::new()),
                    None,
                    SupervisionStrategy::Resume);
                let root_safe = tsafe!(root);

                let dlp = tsafe!(ActorPath::new("deadLetters", Some(self.root_path.clone())));
                let dlm = DeadLetters::new();
                let mut dlc = ActorCell::new(
                    tsafe!(self.clone()),
                    dlp.clone(),
                    tsafe!(SyntheticActor {}),
                    0,
                    dispatcher.clone(),
                    tsafe!(dlm),
                    Some(root_safe),
                    SupervisionStrategy::Resume);
                dlc.stopped = false;

                let boxed_dlc = tsafe!(dlc);


                self.dead_letters = Some(Box::new(LocalActorRef::new(boxed_dlc.clone(), dlp)));
                self.dispatchers.lock().unwrap().insert(String::from(name), dispatcher);
            },
            _ => {
                let name_str = String::from(name);
                let mut dispatchers = self.dispatchers.lock().unwrap();

                if !dispatchers.contains_key(&name_str) {
                    dispatchers.insert(name_str, dispatcher);
                } else {
                    panic!("Try to add dispatcher with existed name '{}'", name_str)
                }
            }
        }
    }

    /// Returns dispatcher by name
    fn get_dispatcher(&self, name: &str) -> TSafe<Dispatcher + Send> {
        let d = self.dispatchers.lock().unwrap();
        let d = d.get(&String::from(name));
        if d.is_some() {
            (*d.as_ref().unwrap()).clone()
        } else {
            panic!("Attempt to get not registered dispatcher '{}'", name)
        }
    }

    fn get_dispatchers(&self) -> TSafe<HashMap<String, TSafe<Dispatcher + Send>>> {
        self.dispatchers.clone()
    }

    /// Returns dispatcher by name as executor
    fn get_executor(&self, name: &str) -> TSafe<Executor + Send> {
        tsafe!(WrappedDispatcher::new(self.get_dispatcher(name)))
    }

    fn get_nid(&mut self) -> String {
        let mut nids = self.nids.lock().unwrap();
        let name = nids.to_string();
        *nids = *nids + 1;

        name
    }
}

impl Clone for NetworkActorSystem {
    fn clone(&self) -> Self {

        let dead_letter: Option<ActorRef> = match &self.dead_letters {
            Some(v) =>
                Some((*v).clone()),
            None =>
                None
        };

        let root: Option<TSafe<ActorCell>> = match &self.root {
            Some(v) =>
                Some((*v).clone()),
            None =>
                None
        };

        NetworkActorSystem {
            nids: self.nids.clone(),
            dispatchers: self.dispatchers.clone(),
            dead_letters: dead_letter, //self.dead_letters.clone()
            root,
            root_path: self.root_path.clone(),
            scheduler: self.scheduler.clone(),
            watcher: self.watcher.clone(),
            controller: self.controller.clone()
        }
    }
}

// ============================================================================


#[derive(Clone)]
pub struct ServerNetController {
    acceptor: Acceptor,
    connections: TSafe<HashMap<u32, ServerConnection>>,
    id_counter: TSafe<IdCounter>,

    /// Larid map with actor path key
    larid_path_map: TSafe<HashMap<String, Larid>>,

    /// Larid map with id key
    larid_id_map: TSafe<HashMap<u32, Larid>>,

    /// Serializer for messages which transmitted through network
    messages_serializer: TSafe<MessagesSerializer + Send>,

    /// Deatch with of larid tables
    larid_watcher: TSafe<ActorRef>,

    /// Host actor system
    system: TSafe<NetworkActorSystem>
}

impl ServerNetController {
    pub fn new(addr: SocketAddr, system: TSafe<NetworkActorSystem>, messages_serializer: TSafe<MessagesSerializer + Send>) -> ServerNetController {
        let (s, r) = mpsc::channel();
        let connections = tsafe!(HashMap::new());
        let larid_path_map = tsafe!(HashMap::new());
        let larid_id_map = tsafe!(HashMap::new());
        let id_counter = tsafe!(IdCounter::new(0));
        let larid_watcher = tsafe!(system.lock().unwrap().actor_of(LaridWatcher::props(larid_path_map.clone(), larid_id_map.clone()), Some("larid_watcher")));

        let obj = ServerNetController {
            acceptor: Acceptor::new(addr, s),
            connections: connections.clone(),
            id_counter: id_counter.clone(),
            larid_id_map: larid_id_map.clone(),
            larid_path_map: larid_path_map.clone(),
            messages_serializer: messages_serializer.clone(),
            larid_watcher: larid_watcher.clone(),
            system: system.clone()
        };

        let boxed_self = tsafe!(obj.clone());

        Self::serve(connections, r, system, messages_serializer, boxed_self, larid_path_map, larid_id_map, id_counter, larid_watcher);

        obj
    }

    fn serve(connections: TSafe<HashMap<u32, ServerConnection>>,
             receiver: Receiver<(ServerConnection, Receiver<ConnectionData>)>,
             system: TSafe<NetworkActorSystem>,
             messages_serializer: TSafe<MessagesSerializer + Send>,
             boxed_self: TSafe<Self>,
             larid_path_map: TSafe<HashMap<String, Larid>>,
             larid_id_map: TSafe<HashMap<u32, Larid>>,
             id_counter: TSafe<IdCounter>,
             larid_watcher: TSafe<ActorRef>)
    {
        let connections = connections.clone();
        let mut cid_counter = 200;
        let mut rid_counter = 100;

        thread::spawn(move || {

            loop {
                let rcv = receiver.recv();
                if rcv.is_err() {
                    break;
                }

                let (connection, data_receiver) = rcv.unwrap();

                let cid = {
                    let mut connections = connections.lock().unwrap();
                    let cid = cid_counter;
                    connections.insert(cid, connection);
                    cid_counter = cid_counter + 1;

                    cid
                };

                let connections = connections.clone();
                let system = system.clone();
                let messages_serializer = messages_serializer.clone();
                let boxed_self = boxed_self.clone();
                let id_counter = id_counter.clone();
                let larid_id_map = larid_id_map.clone();
                let larid_path_map = larid_path_map.clone();
                let larid_watcher = larid_watcher.clone();

                thread::spawn(move || {
                    loop {
                        let cd = data_receiver.recv().unwrap();

                        match cd {
                            ConnectionData::ReceivedPacket(packet) => {
                                //println!("{}", packet);

                                match packet.opm {
                                    Opm::ActorSelect => {
                                        let selection = {
                                            let mut system = system.lock().unwrap();
                                            system.actor_select(&packet.body_as_actor_of())
                                        };

                                        let mut rids = Vec::new();

                                        {

                                            for r in selection {
                                                // Obtain larid
                                                let larid = {
                                                    let path = r.path().to_string();
                                                    let id = id_counter.lock().unwrap().get_id();
                                                    let mut larid_path_map = larid_path_map.lock().unwrap();
                                                    let larid = larid_path_map.get(&path);
                                                    if larid.is_some() {
                                                        larid.as_ref().unwrap().id
                                                    } else {
                                                        let sender = r;
                                                        let larid_watcher = larid_watcher.lock().unwrap();
                                                        system.lock().unwrap().watch(&larid_watcher.clone(), &sender);

                                                        larid_path_map.insert(path, Larid { id, aref: sender.clone() });
                                                        let mut larid_id_map = larid_id_map.lock().unwrap();
                                                        larid_id_map.insert(id, Larid { id, aref: sender });
                                                        id
                                                    }
                                                };

                                                rids.push(larid);
                                            }
                                        }

                                        let mut connections = connections.lock().unwrap();
                                        let mut connection = connections.get_mut(&cid).unwrap();
                                        connection.send(Packet::new_actor_of_response(packet.id, rids));
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

                                        let larid_id_map = larid_id_map.lock().unwrap();
                                        let aref = larid_id_map.get(&larid);

                                        if aref.is_none() {
                                            error!("Unable to receive message from remote system doe to target actor is died");
                                            // Reply with delivery error
                                            // ...................................
                                            continue;
                                        }
                                        let path = ActorPath::new(&format!("remote://{}", &rarid), None);
                                        let sender: ActorRef = Box::new(RemoteActorRef::new(cid, rarid, tsafe!(path), boxed_self.clone()));
                                        let mut aref = (**aref.as_ref().unwrap()).aref.clone();
                                        aref.tell(msg, Some(&sender));
                                    },
                                    e => error!("Handled remote packet with unsupported opm '{:?}'", e)
                                }
                            },
                            ConnectionData::Closed => break
                        }
                    }

                    trace!("Server connection processor was stopped");
                });
            }

            trace!("Server main processor was stopped");
        });
    }

    pub fn stop(&mut self) {
        let mut connections = self.connections.lock().unwrap();

        for (k, conn) in connections.iter_mut() {
            conn.close();
        }

        connections.clear();

        self.larid_path_map.lock().unwrap().clear();
        self.larid_id_map.lock().unwrap().clear();
        self.system.lock().unwrap().stop(&mut self.larid_watcher.lock().unwrap().clone());
        self.acceptor.stop();
    }
}

impl NetController for ServerNetController {
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
                let id = self.id_counter.lock().unwrap().get_id();
                let mut larid_path_map = self.larid_path_map.lock().unwrap();
                let larid = larid_path_map.get(&path);
                if larid.is_some() {
                    larid.as_ref().unwrap().id
                } else {
                    let sender = (*sender.as_ref().unwrap()).clone();
                    let larid_watcher = self.larid_watcher.lock().unwrap();
                    self.system.lock().unwrap().watch(&larid_watcher.clone(), &sender);

                    larid_path_map.insert(path, Larid { id, aref: sender.clone() });
                    let mut larid_id_map = self.larid_id_map.lock().unwrap();
                    larid_id_map.insert(id, Larid { id, aref: sender });
                    id
                }
            } else {
                0
            }
        };

        // Create packet
        let id = self.id_counter.lock().unwrap().get_id();
        let packet = Packet::new_send_msg(id, bin.marker, &bin.blob[..], rarid, larid);


        // Get connection
        let mut connections = self.connections.lock().unwrap();
        let connection = connections.remove(&rcid);

        if connection.is_none() {
            error!("Unable to send message to remote system due to lost connection [ rcid={} ]", rcid);
            if sender.is_some() {
                let mut sender = (*sender.as_ref().unwrap()).clone();
                sender.tell(msg!(DeliveryError { reason: DeliveryErrorReason::ConnectionLost}), Some(&far));
            }
            return;
        }
        let mut connection = connection.unwrap();
        let result = connection.send(packet);
        if !result {
            error!("Unable to send message to remote system due to network error");
            if sender.is_some() {
                let mut sender = (*sender.as_ref().unwrap()).clone();
                sender.tell(msg!(DeliveryError { reason: DeliveryErrorReason::NetworkError}), Some(&far));
            }
        }
        connections.insert(rcid, connection);
    }
}

impl Drop for NetworkActorSystem {
    fn drop(&mut self) {
        println!("NetworkActorSystem dropped")
    }
}


impl Drop for ServerNetController {
    fn drop(&mut self) {
        println!("ServerNetController dropped")
    }
}