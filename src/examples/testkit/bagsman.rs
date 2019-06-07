use crate::actors::actor::Actor;
use crate::actors::actor_context::ActorContext;
use crate::actors::props::Props;
use crate::actors::abstract_actor_ref::ActorRef;
use crate::examples::actors::logger::logger;
use std::any::Any;
use std::sync::{Mutex, Arc};
use match_downcast::*;


pub fn props(referal: Option<ActorRef>) -> Props {
    Props::new(tsafe!(BagsMan::new(referal)))
}

mod commands {
    pub struct MsgOk { pub data: u32 }
    pub struct MsgOk2 { pub data: u32 }
    pub struct MsgOk3 { pub data: u32 }
    pub struct MsgOk4 { pub data: u32 }
    pub struct MsgOk5 { pub data: u32 }
    pub struct MsgOther { pub data: u32 }
    pub struct MsgComplex0 { }
    pub struct MsgComplex1 { }
    pub struct MsgNoResponse { pub data: u32 }
    pub struct ToRef { }
    pub struct MutState { pub data: u32 }
}

mod responses {
    pub struct MsgResponse { pub  data: u32 }
    pub struct MsgResponse2 { pub  data: u32 }
    pub struct MsgResponse3 { pub  data: u32 }
    pub struct MsgResponse4 { pub  data: u32 }
    pub struct OtherResponse { pub  data: u32 }
}


pub struct BagsMan {
    referal: Option<ActorRef>,
    data: u32
}

impl BagsMan {
    pub fn new(referal: Option<ActorRef>) -> BagsMan {
        BagsMan {
            referal,
            data: 0
        }
    }
}

impl Actor for BagsMan {

    fn receive(self: &mut Self, msg: &Box<Any + Send>, mut ctx: ActorContext) -> bool {
        match_downcast_ref!(msg, {
            m: commands::MsgOk => {
                ctx.sender.tell(Box::new(responses::MsgResponse {data: 99} ), Some(ctx.self_.clone()))
            },
            m: commands::MsgOk2 => {
                ctx.sender.tell(Box::new(responses::MsgResponse2 {data: 99} ), Some(ctx.self_.clone()))
            },
            m: commands::MsgOk3 => {
                ctx.sender.tell(Box::new(responses::MsgResponse4 {data: 99} ), Some(ctx.self_.clone()))
            },
            m: commands::MsgOk4 => {
                ctx.sender.tell(Box::new(responses::MsgResponse {data: 99} ), Some(ctx.self_.clone()));
                ctx.sender.tell(Box::new(responses::MsgResponse2 {data: 99} ), Some(ctx.self_.clone()));
                ctx.sender.tell(Box::new(responses::MsgResponse3 {data: 99} ), Some(ctx.self_.clone()))
            },
            m: commands::MsgOk5 => {
                ctx.sender.tell(Box::new(responses::MsgResponse {data: 99} ), Some(ctx.self_.clone()));
                ctx.sender.tell(Box::new(responses::MsgResponse2 {data: 99} ), Some(ctx.self_.clone()));
                ctx.sender.tell(Box::new(responses::OtherResponse {data: 99} ), Some(ctx.self_.clone()))
            },
            m: commands::MsgOther => {
                // Respond with incorrect message
                ctx.sender.tell(Box::new(responses::OtherResponse {data: 99} ), Some(ctx.self_.clone()))
            },
            m: commands::MsgNoResponse => {
                // Does not respond
            },
            m: commands::MsgComplex0 => {
                ctx.sender.tell(Box::new(responses::MsgResponse {data: 99} ), Some(ctx.self_.clone()))
            },
            m: commands::MsgComplex1 => {
                ctx.sender.tell(Box::new(responses::MsgResponse2 {data: 199} ), Some(ctx.self_.clone()))
            },
            m: commands::MutState => {
                self.data = m.data;
            },
            m: commands::ToRef => {
                let msg = Box::new(logger::Log { text: String::from("Text"), target: logger::LogTarget::File });
                if let Some(ref mut x) = self.referal {
                    x.tell(msg, Some(ctx.self_.clone()))
                }
            },
            _ => return false
        });

        true
    }

    fn as_any(self: &Self) -> &Any { self }
}



#[cfg(test)]
mod tests {
    use crate::testkit::actors::test_local_actor_ref::TestLocalActorRef;
    use crate::testkit::actors::test_local_actor_system::TestLocalActorSystem;
    use crate::actors::actor_ref_factory::ActorRefFactory;
    use crate::actors::abstract_actor_ref::AbstractActorRef;
    use crate::examples::actors::logger::logger;
    use std::any::Any;
    use std::time::Duration;
    use super::*;




    #[test]
    fn main() {
        let mut system = TestLocalActorSystem::new();

        // ================================ expect_msg ================================

        test_case!("MsgOk - respond with MsgResponse");
        {
            let (mut target, mut probe) = {
                let mut system = system.lock().unwrap();
                let mut target = system.actor_of(self::props(None), None);
                let mut probe = system.create_probe(Some("probe"));

                (target, probe)
            };

            probe.send(target.clone(), Box::new( commands::MsgOk { data: 99 } ));
            probe.expect_msg(pat_matcher!(responses::MsgResponse => responses::MsgResponse { data: 99 }));
        }

        // This test will failed, uncomment for enable
//        test_case!("MsgOther - respond with OtherResponse");
//        {
//            let (mut target, mut probe) = {
//                let mut system = system.lock().unwrap();
//                let mut target = system.actor_of(self::props(), None);
//                let mut probe = system.create_probe(Some("probe"));
//
//                (target, probe)
//            };
//
//            probe.send(target.clone(), Box::new( commands::MsgOther { data: 99 } ));
//
//            // Oops! In this expectation the probe will receive unexpected message
//            probe.expect_msg(pat_matcher!(responses::MsgResponse => responses::MsgResponse { data: 99 }));
//        }

        // This test will failed, uncomment for enable
//        test_case!("MsgNoResponse - respond with OtherResponse");
//        {
//            let (mut target, mut probe) = {
//                let mut system = system.lock().unwrap();
//                let mut target = system.actor_of(self::props(), None);
//                let mut probe = system.create_probe(Some("probe"));
//
//                (target, probe)
//            };
//
//            probe.send(target.clone(), Box::new( commands::MsgNoResponse { data: 99 } ));
//
//            // Oops! Actor does not respond with expected timeout
//            probe.expect_msg(pat_matcher!(responses::OtherResponse => responses::OtherResponse { data: 99 }));
//        }

        // ================================ expect_any_of ================================

        test_case!("MsgOk2 - respond with any of <MsgResponse, MsgResponse2, MsgResponse3>");
        {
            let (mut target, mut probe) = {
                let mut system = system.lock().unwrap();
                let mut target = system.actor_of(self::props(None), None);
                let mut probe = system.create_probe(Some("probe"));

                (target, probe)
            };

            probe.send(target.clone(), Box::new( commands::MsgOk2 { data: 99 } ));
            probe.expect_msg_any_of(
                vec![
                    type_matcher!(responses::MsgResponse),
                    type_matcher!(responses::MsgResponse2),
                    type_matcher!(responses::MsgResponse3)
                ]
            );
        }

        // This test will failed, uncomment for enable
//        test_case!("MsgOk3 - respond with any of <MsgResponse, MsgResponse2, MsgResponse3>");
//        {
//            let (mut target, mut probe) = {
//                let mut system = system.lock().unwrap();
//                let mut target = system.actor_of(self::props(), None);
//                let mut probe = system.create_probe(Some("probe"));
//
//                (target, probe)
//            };
//
//            probe.send(target.clone(), Box::new( commands::MsgOk3 { data: 99 } ));
//
//            //Oops! Actor responds with message MsgResponse4 which does not in the set
//            probe.expect_msg_any_of(
//                vec![
//                    type_matcher!(responses::MsgResponse),
//                    type_matcher!(responses::MsgResponse2),
//                    type_matcher!(responses::MsgResponse3)
//                ]
//            );
//        }

        // This test will failed, uncomment for enable
//        test_case!("MsgNoResponse - respond with any of <MsgResponse, MsgResponse2, MsgResponse3>");
//        {
//            let (mut target, mut probe) = {
//                let mut system = system.lock().unwrap();
//                let mut target = system.actor_of(self::props(), None);
//                let mut probe = system.create_probe(Some("probe"));
//
//                (target, probe)
//            };
//
//            probe.send(target.clone(), Box::new( commands::MsgNoResponse { data: 99 } ));
//
//            // Oops! Actor does not respond with any messages with expected timeout
//            probe.expect_msg(pat_matcher!(responses::OtherResponse => responses::OtherResponse { data: 99 }));
//        }

        // ================================ expect_all_of ================================

        test_case!("MsgOk4 - respond with all of <MsgResponse, MsgResponse2, MsgResponse3>");
        {
            let (mut target, mut probe) = {
                let mut system = system.lock().unwrap();
                let mut target = system.actor_of(self::props(None), None);
                let mut probe = system.create_probe(Some("probe"));

                (target, probe)
            };

            probe.send(target.clone(), Box::new( commands::MsgOk4 { data: 99 } ));
            probe.expect_msg_all_of(
                vec![
                    type_matcher!(responses::MsgResponse),
                    type_matcher!(responses::MsgResponse2),
                    type_matcher!(responses::MsgResponse3)
                ]
            );
        }

        // This test will failed, uncomment for enable
//        test_case!("MsgOk5 - respond with all of <MsgResponse, MsgResponse2, MsgResponse3>");
//        {
//            let (mut target, mut probe) = {
//                let mut system = system.lock().unwrap();
//                let mut target = system.actor_of(self::props(None), None);
//                let mut probe = system.create_probe(Some("probe"));
//
//                (target, probe)
//            };
//
//            probe.send(target.clone(), Box::new( commands::MsgOk5 { data: 99 } ));
//
//            // Oops! Not all messages will be expected. This situation case timeout and it is correct!
//            // It determines that the specified messages will be guaranteed from the stream, in what
//            // order, with which intermediate messages it does not matter.
//            probe.expect_msg_all_of(
//                vec![
//                    type_matcher!(responses::MsgResponse),
//                    type_matcher!(responses::MsgResponse2),
//                    type_matcher!(responses::MsgResponse3)
//                ]
//            );
//        }

        // This test will failed, uncomment for enable
//        test_case!("MsgNoResponse - respond with any of <MsgResponse, MsgResponse2, MsgResponse3>");
//        {
//            let (mut target, mut probe) = {
//                let mut system = system.lock().unwrap();
//                let mut target = system.actor_of(self::props(None), None);
//                let mut probe = system.create_probe(Some("probe"));
//
//                (target, probe)
//            };
//
//            probe.send(target.clone(), Box::new( commands::MsgNoResponse { data: 99 } ));
//
//            // Oops! Actor does not respond with all messages with expected timeout
//            probe.expect_msg_all_of(
//                vec![
//                    type_matcher!(responses::MsgResponse),
//                    type_matcher!(responses::MsgResponse2),
//                    type_matcher!(responses::MsgResponse3)
//                ]
//            );
//        }

        // ================================ expect_no_msg ================================

        test_case!("MsgNoResponse - expect no message");
        {
            let (mut target, mut probe) = {
                let mut system = system.lock().unwrap();
                let mut target = system.actor_of(self::props(None), None);
                let mut probe = system.create_probe(Some("probe"));

                (target, probe)
            };

            probe.send(target.clone(), Box::new( commands::MsgNoResponse { data: 99 } ));
            probe.expect_no_msg(Duration::from_secs(1));
        }

        // This test will failed, uncomment for enable
//        test_case!("MsgOther - not respond for any messages");
//        {
//            let (mut target, mut probe) = {
//                let mut system = system.lock().unwrap();
//                let mut target = system.actor_of(self::props(), None);
//                let mut probe = system.create_probe(Some("probe"));
//
//                (target, probe)
//            };
//
//            probe.send(target.clone(), Box::new( commands::MsgOther { data: 99 } ));
//
//            // Oops! In this expectation the probe should not have received any message, but received
//            probe.expect_no_msg(Duration::from_secs(1));
//        }

        // ================================ complex chain ================================

        // This test demonstrate complex messages testing. We send message to the actor, expect
        // response from it, reply him and expect again some message.

        test_case!("Process complex logic");
        {
            let (mut target, mut probe) = {
                let mut system = system.lock().unwrap();
                let mut target = system.actor_of(self::props(None), None);
                let mut probe = system.create_probe(Some("probe"));

                (target, probe)
            };

            probe.send(target.clone(), Box::new( commands::MsgComplex0 {  } ));
            probe.expect_msg(pat_matcher!(responses::MsgResponse => responses::MsgResponse { data: 99 }));
            probe.reply(Box::new( commands::MsgComplex1 {  } ));
            probe.expect_msg(pat_matcher!(responses::MsgResponse2 => responses::MsgResponse2 { data: 199 }));
        }


        // ================================ injection tests ================================

        // Referral testing - test case when you inject test probe actor as regular the target actor
        // dependency. In this case, all message passed from the actor to the referal actor, will by
        // intercepted by testprobe.

        test_case!("Send message to the referal actor");
        {
            let (mut target, mut probe) = {
                let mut system = system.lock().unwrap();
                let mut probe = system.create_probe(Some("probe"));

                // See to this code - we extract internal testprobe actor, and pass it as target
                // actor dependency
                let mut target = system.actor_of(self::props(Some(probe.aref())), None);

                (target, probe)
            };

            probe.send(target.clone(), Box::new(commands::ToRef {}));
            probe.expect_msg(type_matcher!(logger::Log));
            // And here you may reply to this message. Target actor receive is as from the referal actor
        }

        // ================================ access to the actor state ================================

        test_case!("Get internal state");
        {
            let (mut target, mut probe) = {
                let mut system = system.lock().unwrap();
                let mut probe = system.create_probe(Some("probe"));

                // See to this code - we extract internal testprobe actor, and pass it as target
                // actor dependency
                let mut target = system.actor_of(self::props(Some(probe.aref())), None);

                (target, probe)
            };

            probe.send(target.clone(), Box::new(commands::MutState { data: 599 }));

            // Wait while message will reach the actor and mutate his state
            probe.expect_no_msg(Duration::from_millis(500));


            in_state! (target, BagsMan, actor => {
                assert_eq!(actor.data, 599);
            });
        }



        // ================================ various matchers ================================

        // This section simply demonstrates, how various matchers functions may be constructed


        // Match my message type
        let type_matcher = type_matcher!(logger::Log);

        // Match by pattern (without guards)
        let pat_mather = pat_matcher!(logger::Log => logger::Log { text: _, target: logger::LogTarget::StdOut });

        // Match by type with extended clarifications
        let extended_matcher = extended_type_matcher!(logger::Log, v => {
            if v.text.len() > 100 {
                true
            } else {
                false
            }
        });

        // Flat matcher ( sweetened version of the raw matcher function )
        let flat_matcher = matcher!(v => {
            if let Some(m) = v.downcast_ref::<logger::Log>() {
                if m.text.len() > 100 {
                    match m.target {
                        logger::LogTarget::StdOut => true,
                        _ => false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        });

        // Raw matcher function, without any syntactic sugar
        let raw_matcher = Box::new(|v: &Box<Any + Send>| {
            if let Some(m) = v.downcast_ref::<logger::Log>() {
                if m.text.len() > 100 {
                    match m.target {
                        logger::LogTarget::StdOut => true,
                        _ => false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        });
    }
}