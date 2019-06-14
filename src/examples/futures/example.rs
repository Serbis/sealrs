use crate::futures::future::{Future, WrappedFuture};
use crate::futures::promise::Promise;
use crate::futures::completable_promise::CompletablePromise;
use crate::futures::async_promise::AsyncPromise;
use crate::common::tsafe::TSafe;
use crate::executors::thread_pinned_executor::ThreadPinnedExecutor;
use std::sync::{Arc, Mutex};
use failure::Fail;
use std::thread;
use std::time::Duration;

#[derive(Debug, Fail)]
enum MyError {
    #[fail(display = "my custom error: {}", text)]
    ExampleError {
        text: String,
    }
}


pub fn run() {
    //completable_promise();
    //async_promise();
    //future_combinators();
    future_await();

    thread::park();
}

fn completable_promise() {
    let mut p: CompletablePromise<u32, TSafe<Fail + Send>> = CompletablePromise::new();
    let mut fut = p.future();

    fut.on_complete(|v| {
        // This code will be called only after that, when future will be filled by the promise from
        // another thread

        println!("Result={}", v.as_ref().ok().unwrap());

        // In this place you may do some very more complicated work than simple result printing. For
        // example you may send an http request to a remote server, based on the result of the
        // calculation.
    });

    thread::spawn(move || {
        // Complete promise
        p.success(100);
    });
}

fn async_promise() {
    let mut executor = tsafe!(ThreadPinnedExecutor::new().run());

    let mut p: AsyncPromise<u32, TSafe<Fail + Send>> =
        AsyncPromise::new(Box::new(|| Ok(50 + 50)), executor);

    let mut fut = p.future();

    // Or shortly
    // let mut fut: WrappedFuture<u32, TSafe<Fail + Send>> =
    //      Future::asyncp(|| Ok(500 + 500), executor)

    fut.on_complete(|v| {
        // This code will be called only after that, when future will be filled by the promise when
        // it be automatically completed with result of closure execution

        println!("Result={}", v.as_ref().ok().unwrap());

        // In this place you may do some very more complicated work than simple result printing. For
        // example you may send an http request to a remote server, based on the result of the
        // calculation.
    });
}

fn future_combinators() {

    let mut executor = tsafe!(ThreadPinnedExecutor::new().run());

    // map

    let mut fut0: WrappedFuture<u32, TSafe<Fail + Send>> =
          Future::asyncp(|| Ok(500 + 500), executor.clone());

    fut0.map(|v| {
        // Transform value from u32 to String
        Ok(format!("{}", v))
    });


    // flat_map


    let mut fut1: WrappedFuture<u32, TSafe<Fail + Send>> =
        Future::asyncp(|| Ok(500 + 500), executor.clone());

    let mut executor1 = executor.clone();

    fut1.flat_map(move |v| {
        let vc = *v;
        let fut_inner: WrappedFuture<String, TSafe<Fail + Send>> =
            Future::asyncp(move || {
                let r = format!("{}", vc + 1000);
                Ok(r)
            }, executor1.clone());
        Ok(fut_inner)
    });


    // recover


    let mut fut2: WrappedFuture<u32, TSafe<Fail + Send>> =
        Future::asyncp(|| Ok(500 + 500), executor.clone());

    fut2.map(|_| {
        // Oops! Some error occurs in this map
        let err = tsafe!(MyError::ExampleError { text: String::from("Oops!") });
        Err(err)
    }).recover(|_| {
        // Handle error in this place

        // And return 100 as recovery result ( as default value )
        Ok(100)
    });


    // on_complete


    let mut fut3: WrappedFuture<u32, TSafe<Fail + Send>> =
        Future::asyncp(|| Ok(500 + 500), executor.clone());

    fut3.on_complete(|v| {
        match v {
            Ok(result) => println!("Result={}", result),
            Err(error) => println!("Error={}", error.lock().unwrap())
        }
    });


    // chains

    let vst = 1000;

    let mut fut4: WrappedFuture<u32, TSafe<Fail + Send>> =
        Future::asyncp(move || {
            if vst > 500 {
                Ok(vst + 1000)
            } else {
                let err: TSafe<Fail + Send> =
                    tsafe!(MyError::ExampleError { text: String::from("Vst must be > 500!") });
                Err(err)
            }
        }, executor.clone());

    fut4.map(|v| {
        Ok(format!("{}", *v))
    }).recover(|e| {
        println!("Recovered to default 100 because error occurs -> {}", e.lock().unwrap());
        Ok(String::from("100"))
    }).on_complete(|v| {
        println!("Result={}", v.as_ref().unwrap())
    });
}

pub fn future_await() {
    let mut executor = tsafe!(ThreadPinnedExecutor::new().run());

    // Ready

    let mut fut1: WrappedFuture<u32, TSafe<Fail + Send>> =
        Future::asyncp(|| {
            thread::sleep(Duration::from_secs(1));
            Ok(100)
        }, executor.clone());

    let ready = fut1.ready(Duration::from_secs(3));

    match ready {
        true => {
            match fut1.get_value() {
                Ok(value) => {
                    println!("fut1 completed with Ok({})", value);
                },
                Err(error) => {
                    println!("fut1 completed with Err({})", error.lock().unwrap());
                }
            }

        },
        false => {
            println!("fut1 does not completed with timeout");
        }
    }

    // Result

    let mut fut2: WrappedFuture<u32, TSafe<Fail + Send>> =
        Future::asyncp(|| {
            thread::sleep(Duration::from_secs(1));
            Ok(100)
        }, executor.clone());

    let result = fut2.result(Duration::from_secs(3));

    match result {
        Ok(value) => {
            match value {
                Ok(value) => {
                    println!("fut2 completed with Ok({})", value);
                },
                Err(error) => {
                    println!("fut2 completed with Err({})", error.lock().unwrap());
                }
            }
        },
        Err(_) => {
            println!("fut2 does not completed with timeout");
        }
    }

}