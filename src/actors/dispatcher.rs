//NOTOK
use crate::actors::actor_cell::ActorCell;
use crate::actors::envelope::Envelope;
use crate::actors::mailbox::Mailbox;
use crate::actors::actor::Actor;
use crate::executors::executor::Executor;
use crate::common::tsafe::TSafe;

pub trait Dispatcher: Executor  {
    fn dispatch(self: &mut Self,
                cell: TSafe<ActorCell>,
                bid: usize,
                mailbox: TSafe<Mailbox + Send>,
                actor: TSafe<Actor + Send>,
                envelope: Envelope);

    fn obtain_bid(self: &mut Self) -> usize;
}