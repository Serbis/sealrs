use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::watcher::WatchingEvents;
use crate::actors::scheduler::Scheduler;
use crate::actors::dispatcher::Dispatcher;
use crate::common::tsafe::TSafe;
use crate::executors::executor::Executor;
use std::collections::HashMap;

//TODO docs
pub trait AbstractActorSystem: ActorRefFactory {

    /// Returns actor system scheduler
    fn get_scheduler(&self) -> TSafe<Scheduler>;

    /// Register new watching event from the specified actor
    fn register_watch_event(&self, from: &ActorRef, event: WatchingEvents);

    /// Stops the actor system
    fn terminate(&mut self);

    /// Adds new dispatcher to the system. For now supporter only default dispatcher replacing
    fn add_dispatcher(&mut self, name: &str, dispatcher: TSafe<Dispatcher + Send>);

    /// Returns dispatcher by name
    fn get_dispatcher(&self, name: &str) -> TSafe<Dispatcher + Send>;

    /// Returns dispatchers list
    fn get_dispatchers(&self) -> TSafe<HashMap<String, TSafe<Dispatcher + Send>>>;

    /// Returns dispatcher by name as executor
    fn get_executor(&self, name: &str) -> TSafe<Executor + Send>;

    /// Return actor auto name
    fn get_nid(&mut self) -> String;
}

//TODO остановка акторной системы
//TODO вызов PostStop из drop и проверка теории закольцованных ссылко и ручного сброса актора (drop)
