use crate::executors::thread_pinned_executor::{ThreadPinnedExecutor, DistributionStrategy, TaskOptions};
use crate::executors::executor::{Executor,ExecutorTask};
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

pub struct MyDispatcher {
    executor: ThreadPinnedExecutor,
    rounds: usize
}

impl MyDispatcher {
    pub fn new(t_count: u32) -> MyDispatcher {
        let executor = ThreadPinnedExecutor::new()
            .set_distribution_strategy(DistributionStrategy::Load)
            .set_threads_count(t_count as usize)
            .run();

        println!(" - Created new MyDispatcher instance");
        MyDispatcher {
            executor,
            rounds: 0
        }
    }

    pub fn invoke(mailbox: &TSafe<Mailbox + Send>, actor: &TSafe<Actor + Send>, cell: &TSafe<ActorCell>) {
        let envelope = {
            let mut mailbox = mailbox.lock().unwrap();
            if mailbox.has_messages() {
                Some(mailbox.dequeue())
            } else {
                None
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
                println!(" - Handle message with actor");
                actor.receive(msg.clone(), ctx)
            };

            if !handled {
                println!(" - Handle message with internal receive");
                let handled2 = MyDispatcher::internal_receive(mailbox, msg.clone(), cell);
                if !handled2 {
                    let mut dead_letters = {
                        let mut system = envelope.system.lock().unwrap();
                        let dead_letters = system.dead_letters();
                        dead_letters
                    };
                    println!(" - Message does not handled, drain to DeadLetters");
                    dead_letters.cell().lock().unwrap().send(&dead_letters.cell(), msg, Some(sender), envelope.receiver );

                }
            }
        }

        mailbox.lock().unwrap().set_planned(false);
    }

    pub fn internal_receive(mailbox: &TSafe<Mailbox + Send>, msg: Message, cell: &TSafe<ActorCell>) -> bool {

        if let Some(PoisonPill {}) = msg.get().downcast_ref::<PoisonPill>() {
            println!(" - Handled PoisonPill message. Start actor termination procedure");
            let mut cell_u = cell.lock().unwrap();
            cell_u.suspend();
            let dead_letters = cell_u.system.lock().unwrap().dead_letters();
            mailbox.lock().unwrap().clean_up(Box::new(LocalActorRef::new(cell.clone(), cell_u.path.clone())), dead_letters);
            cell_u.stop(cell.clone());
        } else {
            return false
        }

        true
    }


}

impl Executor for MyDispatcher {
    fn execute(&mut self, f: ExecutorTask, options: Option<Box<Any>>) {
        println!(" - Plan task for execution");
        self.executor.execute(f, options)
    }

    fn stop(&mut self) {
        println!(" - Dispatcher was stopped");
        self.executor.stop();
    }
}

impl Dispatcher for MyDispatcher {

    fn dispatch(self: &mut Self, cell: TSafe<ActorCell>, bid: usize, mailbox: TSafe<Mailbox + Send>, actor: TSafe<Actor + Send>, envelope: Envelope) {
        let mut mailbox_u = mailbox.lock().unwrap();
        mailbox_u.enqueue(envelope);

        let mailbox = mailbox.clone();
        let f = Box::new(move || {
            MyDispatcher::invoke(&mailbox, &actor, &cell)
        });

        println!(" - Dispatch new message for bid {}", &bid);
        self.execute(f,  Some( Box::new(TaskOptions { thread_id: Some(bid) } )))
    }

    fn obtain_bid(self: &mut Self) -> usize {
        if self.rounds == self.executor.get_threads_count() - 1 {
            self.rounds = 0;
        } else {
            self.rounds = self.rounds + 1;
        }

        println!(" - Obtained new bid {}", self.rounds);

        self.rounds
    }
}