use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::actors::watcher::WatchingEvents;
use crate::actors::scheduler::Scheduler;
use crate::common::tsafe::TSafe;

//TODO docs
pub trait AbstractActorSystem: ActorRefFactory {

    /// Returns actor system scheduler
    fn get_scheduler(&self) -> TSafe<Scheduler>;

    /// Register new watching event from the specified actor
    fn register_watch_event(&self, from: &ActorRef, event: WatchingEvents);
}

//TODO остановка акторной системы
//TODO вызов PostStop из drop и проверка теории закольцованных ссылко и ручного сброса актора (drop)
