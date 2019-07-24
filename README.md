
# About

Sealrs is set of various modules for highly concurrent applications, developing under strongly impact from the titans of the asynchronous programming world, such us Erlang, Scala and Akka.

Library includes next submodules:
* [actors](https://docs.rs/sealrs/*/sealrs/actors/index.html) - Actor-based concurrent runtime, based on the untyped actors and paradigms which actively used in Akka framework and Erlang language.
* [futures](https://docs.rs/sealrs/*/sealrs/futures/index.html) - Future-based runtime based on the classic computer-science definition of 'Future/Promise' paradigm. ( under developing )
* [executors](https://docs.rs/sealrs/*/sealrs/executors/index.html) - Set of various concurrent executors, actively used by other modules of the library, and which may be used by the user.
* [testkit](https://docs.rs/sealrs/*/sealrs/testkit/index.html) - Test framework for deep and seamless testing of code developed based on this library.

This library have a very reach documentation with big count of examples and explanations of internal library architecture. Read on the [docs.rs](https://docs.rs/sealrs/).

# New in release

* Changed data mutability in FSM
* Fixed bug with deadlock of dispatcher thread if message was unhandled
* Added stop method to TestProbe which allows to stop internal actor
* Added get_last_sender method to TestProbe
* Realized basic remoting (actor_select, message passing)

See [changelog](https://github.com/Serbis/sealrs/blob/master/changelog.md) for info about new releases.

# In the next release

* Additional remoting feature

# Developing stage info

At the moment, I have implemented almost all the necessary functionality for using this library as a framework for building web applications. In the short term, I do not plan strategic extensions of the functionality of this library with the exception of the expansion of existing functions and bug fixes.