//! Actor dispatcher with strategy - dedicated thread per actor
use crate::executors::thread_pinned_executor::{ThreadPinnedExecutor, DistributionStrategy, TaskOptions};
use crate::executors::executor::{Executor, ExecutorTask};
use crate::actors::dispatcher::Dispatcher;
use crate::actors::actor_cell::ActorCell;
use crate::actors::local_actor_ref::LocalActorRef;
use crate::actors::abstract_actor_ref::AbstractActorRef;
use crate::actors::actor_context::ActorContext;
use crate::actors::envelope::Envelope;
use crate::actors::mailbox::Mailbox;
use crate::actors::actor::{Actor, PoisonPill};
use crate::actors::message::Message;
use crate::common::tsafe::TSafe;
use std::any::Any;


pub struct PinnedDispatcher {
    executor: ThreadPinnedExecutor,
    rounds: usize
}

impl PinnedDispatcher {
    pub fn new() -> PinnedDispatcher {
        let executor = ThreadPinnedExecutor::new()
            .set_distribution_strategy(DistributionStrategy::EventLoop)
            .set_threads_count(1)
            .run();
        PinnedDispatcher {
            executor,
            rounds: 0
        }
    }

    pub fn invoke(mailbox: &TSafe<Mailbox + Send>, actor: &TSafe<Actor + Send>, cell: &TSafe<ActorCell>) {
        while true {
            let envelope = {
                let mut mailbox = mailbox.lock().unwrap();
                if mailbox.has_messages() {
                    Some(mailbox.dequeue())
                } else {
                    mailbox.set_planned(false);
                    break;
                }
            };

            if envelope.is_some() {
                let envelope = envelope.unwrap();

                let sender: Box<AbstractActorRef + Send> = {
                    if envelope.sender.is_some() {
                        envelope.sender.unwrap()
                    } else {
                        let mut system = envelope.system.lock().unwrap();
                        let dead_letters = system.dead_letters();
                        dead_letters
                    }
                };


                let msg = envelope.message;

                let handled = {
                    let mut actor = actor.lock().unwrap();
                    let ctx = ActorContext::new(
                        sender.clone(),
                        envelope.receiver.clone(),
                        envelope.system.clone());
                    actor.receive(msg.clone(), ctx)
                };

                if !handled {
                    let handled2 = PinnedDispatcher::internal_receive(mailbox, msg.clone(), cell);
                    if !handled2 {
                        let mut dead_letters = {
                            let mut system = envelope.system.lock().unwrap();
                            let dead_letters = system.dead_letters();
                            dead_letters
                        };
                        dead_letters.cell().lock().unwrap().send(&dead_letters.cell(), msg, Some(sender), envelope.receiver );

                    }
                }
            }
        }
    }

    pub fn internal_receive(mailbox: &TSafe<Mailbox + Send>, msg: Message, cell: &TSafe<ActorCell>) -> bool {

        if let Some(PoisonPill {}) = msg.get().downcast_ref::<PoisonPill>() {
            let mut cell_u = cell.lock().unwrap();
            cell_u.suspend();
            // +++ cell.actor.timers().cancelAll();
            let dead_letters = cell_u.system.lock().unwrap().dead_letters();
            mailbox.lock().unwrap().clean_up(Box::new(LocalActorRef::new(cell.clone(), cell_u.path.clone())), dead_letters);
            cell_u.stop(cell.clone());
            cell_u.dispatcher.lock().unwrap().stop();
        } else {
            return false
        }

        true
    }
}

impl Executor for PinnedDispatcher {
    fn execute(&mut self, f: ExecutorTask, options: Option<Box<Any>>) {
        self.executor.execute(f, options)
    }

    fn stop(&mut self) {
        self.executor.stop();
    }
}

impl Dispatcher for PinnedDispatcher {

    fn dispatch(self: &mut Self, cell: TSafe<ActorCell>, bid: usize, mailbox: TSafe<Mailbox + Send>, actor: TSafe<Actor + Send>, envelope: Envelope) {
        let mut mailbox_u = mailbox.lock().unwrap();
        mailbox_u.enqueue(envelope);
        if !mailbox_u.is_planned() {
            mailbox_u.set_planned(true);

            let mailbox = mailbox.clone();
            let f = Box::new(move || {
                PinnedDispatcher::invoke(&mailbox, &actor, &cell)
            });

            self.execute(f,  None)
        }
    }

    fn obtain_bid(self: &mut Self) -> usize {
        0
    }
}

//impl Drop for PinnedDispatcher {
//    fn drop(&mut self) {
//        println!("PinnedDispatcher dropped")
//    }
//}