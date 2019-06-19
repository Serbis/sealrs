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
