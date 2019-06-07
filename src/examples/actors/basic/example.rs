use crate::actors::local_actor_system::LocalActorSystem;
use crate::actors::actor_ref_factory::ActorRefFactory;
use crate::examples::actors::basic::basic_actor;

pub fn run() {
    let mut system = LocalActorSystem::new();

    let mut printer = system.lock().unwrap()
        .actor_of(basic_actor::props(), Some("printer"));

    let msg = Box::new(basic_actor::Print { text: String::from("Hello world!") });
    printer.tell(msg, None);
}