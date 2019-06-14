
# About

Sealrs is set of various modules for highly concurrent applications, developing under strongly impact from the titans of the asynchronous programming world, such us Erlang, Scala and Akka.

Library includes next submodules:
* [actors](https://docs.rs/sealrs/*/sealrs/actors/index.html) - Actor-based concurrent runtime, based on the untyped actors and paradigms which actively used in Akka framework and Erlang language.
* [futures](https://docs.rs/sealrs/*/sealrs/futures/index.html) - Future-based runtime based on the classic computer-science definition of 'Future/Promise' paradigm. ( under developing )
* [executors](https://docs.rs/sealrs/*/sealrs/executors/index.html) - Set of various concurrent executors, actively used by other modules of the library, and which may be used by the user.
* [testkit](https://docs.rs/sealrs/*/sealrs/testkit/index.html) - Test framework for deep and seamless testing of code developed based on this library.

This library have a very reach documentation with big count of examples and explanations of internal library architecture. Read on the [docs.rs](https://docs.rs/sealrs/).

# New in release

* Developed future blocking operations. [DOC](https://docs.rs/sealrs/*/sealrs/futures/index.html#blocking-operations) for info about new releases.
* Presented converter from the sealrs futures to the rust futures [DOC](https://docs.rs/sealrs/*/sealrs/futures/index.html#futures-conversion) for info about new releases.

See [changelog](https://github.com/Serbis/sealrs/blob/master/changelog.md) for info about new releases.

# In the next release

* PinnedDispatcher ( actor per thread dispatch )

# Why experimental?

Comments about why the library in the experimental state. This library while does not used in my any real project. Reason for this in that I need develop basic fetures of actors / futures . Without this, real usage of this library is very difficult. This situation lead to fact, that for now, library probably contain a very big count of bags. And I will don't see it, until I start to use this library for real development. Accordingly this, while the library will stay in experimental state, I does not may give any guarantee, that separate components of the library work correctly.