# 0.13.2

* Added stop method to Stash
* Added get_last_sender method to TestProbe

# 0.13.1

* Fixed bug - when actor has stopped he was does not removed from child's list

# 0.13.0

* Changed data mutability in FSM
* Fixed bug with deadlock of dispatcher thread if message was unhandled
* Added stop method to TestProbe which allows to stop internal probe actor
* Added get_last_sender method to the TestProbe
* Realized basic remoting

# 0.12.2

* Fixed bug with deadlock in watch mechanism

# 0.12.1

* Fixed bug with panic at actor name generating in TestActorSystem

# 0.12.0

* Error and hierarchy supervision
* actor_select

# 0.11.4

# Added automatic PoisonPill handling in FSM

# 0.11.3

* Fixed bug in TestProbe which consist in that, when test code doest not call expectors, it will lead to the dispatcher deadlock

# 0.11.2

* Change in mutability of ActorContext in state handlers of fsm

# 0.11.1

* Bug on crates.io ( previous version does not load properly)

# 0.11.0

* FSM module
* Changed return type of Actor::receive (supervision requirements)

# 0.10.1

* Changes in timers public api

# 0.10.0

* Developed Stash module
* Added actor_of replacement mechanism to TestActorSystem

# 0.9.1

* Added pre completed futures
* Changes in futures converter

# 0.9.0

* Dispatchers adding mechanism
* Dispatchers swapping mechanism
* Dispatchers wrapper for you may use dispatcher as linear executor
* Developed PinnedDispatcher
* Changed actor system constructor. Now he returns flat type instead TSafe

# 0.8.4

* Developed Future::map_err combinator

# 0.8.3

* Fixed bug with threads count selection (library panic at machine with non 4 cpu cores)

# 0.8.2

* Fixed bug in fixing of the previous bug (forgot delete debug println)

# 0.8.1

* Fixed bag in TestProbe::expect_no_msg

# 0.8.0

* Developed future blocking operations
* Presented converter from the sealrs futures to the rust futures

# 0.7.2

* Realized actor system termination

# 0.7.1

* Default Ask timeout changed from 3 seconds to 500 milliseconds to TestLocalActorRef

# 0.7.0

* Refactored message passing mechanism
* Developed ask operation
* Expectors in TestProbe now returns intercepted message/s

# 0.6.1

Small edits in the matchers (testkit) logic (move in closures)

# 0.6.0

* Change public api in parts of ActorRef's passing
* Developed watch event bus
* Upgraded TestProbe (watch, expect_terminated)
* Integrations with debug log's

# 0.5.0

Timers module

# 0.4.0

Developed basic future / promise functional in completable and asynchronous promises. Created basic functional of the futures in part of basic functional combinators (map, flat_map, recovery, on_complete).

# 0.3.0

Realized ThreadPinnedExecutor.

# 0.2.0

Realized basic features of actor's testkit.

# 0.1.1

Correction of docs.

# 0.1.0

Initial release.
