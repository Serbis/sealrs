use crate::actors::actor_ref_factory::ActorRefFactory;

//TODO docs
pub trait AbstractActorSystem: ActorRefFactory {

}

//TODO остановка акторной системы
//TODO вызов PostStop из drop и проверка теории закольцованных ссылко и ручного сброса актора (drop)
