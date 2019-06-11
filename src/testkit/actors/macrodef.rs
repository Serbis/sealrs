//! Macros used with tests

/// Creates TestProbe matcher function from the user specified block of code
///
/// # Examples
///
/// ```
/// matcher!(v => {
///     if let Some(m) = v.downcast_ref::<some_actor::SomeMsg>() {
///         if m.data > 100 {
///             true
///         } else {
///             false
///         }
///     } else {
///         false
///     }
/// });
/// ```
#[macro_export]
macro_rules! matcher {
        ($value:ident => $body:expr) => {
            Box::new(move |$value: &Box<Any + Send>| {
                $body
            })
        };
    }

/// Creates TestProbe matcher function which match specified message type.
///
/// # Examples
///
/// ```
///  type_matcher!(some_actor::SomeMsg);
/// ```
///
#[macro_export]
macro_rules! type_matcher {
        ($t:path) => {
            Box::new(move |v: &Box<Any + Send>| {
                if let Some(_) = v.downcast_ref::<$t>() {
                   true
                } else {
                    false
                }
            })
        };
    }

/// Creates TestProbe matcher function which match specified message by match arm. Match arm must be
/// specified without guards.
///
/// # Examples
///
/// ```
///  type_matcher!(some_actor::SomeMsg => some_actor::SomeMsg { data: 100 });
///
/// // Message type => Match arm
///
/// ```
///
#[macro_export]
macro_rules! pat_matcher {
        ($t:path => $pat:pat) => {
            Box::new(move |v: &Box<Any + Send>| {
                if let Some(m) = v.downcast_ref::<$t>() {
                    match m {
                         $pat => true,
                         _ => false
                    }
                } else {
                    false
                }
            })
        };
    }

/// Creates TestProbe matcher function which match specified message type and if is was success,
/// apply specified user function to the result.
///
/// # Examples
///
/// ```
/// extended_type_matcher!(some_actor::SomeMsg, v => {
///     if v.data > 100 {
///         true
///     } else {
///         false
///     }
/// });
///
/// // Inner function must return bool value
///
/// ```
///
#[macro_export]
macro_rules! extended_type_matcher {
        ($t:path , $v:ident => $body:expr) => {
            Box::new(move |v: &Box<Any + Send>| {
                if let Some($v) = v.downcast_ref::<$t>() {
                   $body
                } else {
                    false
                }
            })
        };
    }

/// Extract actor object from TestActorRef. This object is immutable and may be used for reversion
/// of the internal actor's state. For use this macros, target actor must implement the as_any
/// method from the Actor trait. Without satisfying this condition, macros will cause panic.
///
/// # Examples
///
/// ```
/// in_state! (target, Foo, actor => {
///     assert_eq!(actor.data, 599);
/// });
///
/// // target - TestActorRef
/// // Foo - actor type under actor reference
/// ```
///
#[macro_export]
macro_rules! in_state {
        ($r:ident , $t:path, $a:ident => $e:expr) => {
            {
                let mut target_any = $r.as_any();
                let mut sd = target_any.downcast_ref::<Box<TestLocalActorRef>>().unwrap();
                let mut actor = sd.actor.lock().unwrap();
                let mut actor = actor.as_any();
                let mut $a = actor.downcast_ref::<$t>().unwrap();
                $e;
            }
        };
    }