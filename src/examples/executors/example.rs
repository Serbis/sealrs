use crate::executors::thread_pinned_executor::{ThreadPinnedExecutor, DistributionStrategy, TaskOptions};
use crate::executors::executor::Executor;
use std::thread;

pub fn run() {
    thread_pinned_dispatcher();
}

fn thread_pinned_dispatcher() {
        let mut executor = ThreadPinnedExecutor::new()
            .set_threads_count(8)
            .set_distribution_strategy(DistributionStrategy::Load)
            .run();

        let f0 = Box::new( || { println!("Task on implicitly selected thread") });
        executor.execute(f0, None);

        let f1 = Box::new( || { println!("Task on explicitly selected thread with id 6") });
        executor.execute(f1, Some( Box::new(TaskOptions { thread_id: Some(6) } )));

    thread::sleep_ms(500);
}