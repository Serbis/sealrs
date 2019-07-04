use crate::actors::props::Props;
use crate::actors::abstract_actor_ref::ActorRef;

//TODO docs
pub trait ActorRefFactory {
    fn actor_of(self: &mut Self, props: Props, name: Option<&str>) -> ActorRef;
    fn actor_select(&mut self, path: &str) -> Vec<ActorRef>;
    fn stop(self: &mut Self, aref: &mut ActorRef);
    fn dead_letters(self: &mut Self) -> ActorRef;

    /// Register watcher for receive 'watching events' from observed actor
    fn watch(&mut self, watcher: &ActorRef, observed: &ActorRef);

    /// Unregister watcher from receive 'watching events' from observed actor
    fn unwatch(&mut self, watcher: &ActorRef, observed: &ActorRef);
}