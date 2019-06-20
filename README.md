
# About

Sealrs is set of various modules for highly concurrent applications, developing under strongly impact from the titans of the asynchronous programming world, such us Erlang, Scala and Akka.

Library includes next submodules:
* [actors](https://docs.rs/sealrs/*/sealrs/actors/index.html) - Actor-based concurrent runtime, based on the untyped actors and paradigms which actively used in Akka framework and Erlang language.
* [futures](https://docs.rs/sealrs/*/sealrs/futures/index.html) - Future-based runtime based on the classic computer-science definition of 'Future/Promise' paradigm. ( under developing )
* [executors](https://docs.rs/sealrs/*/sealrs/executors/index.html) - Set of various concurrent executors, actively used by other modules of the library, and which may be used by the user.
* [testkit](https://docs.rs/sealrs/*/sealrs/testkit/index.html) - Test framework for deep and seamless testing of code developed based on this library.

This library have a very reach documentation with big count of examples and explanations of internal library architecture. Read on the [docs.rs](https://docs.rs/sealrs/).

# New in release

* Developed Stash module
* Added actor_of replacement mechanism to TestActorSystem

See [changelog](https://github.com/Serbis/sealrs/blob/master/changelog.md) for info about new releases.

# In the next release

* PinnedDispatcher ( actor per thread dispatch )

# Developing stage info

Starting with release 0.9.0 the library is translated from experimental to actively-developed stage. This change due to that main functionality of the library is developed and fixed most of critical bugs. Fow now this library is used for developing real microservices in my own project. Despite that fact, public api is not fully stable, and may be partially changed in future releases.