use std::{thread::sleep, time::Duration};
use thread_pool::ThreadPool;

#[test]
fn execute_tasks() {
    let pool = ThreadPool::new(5);

    pool.execute(|| {
        for index in 0..100 {
            println!("In Task1 [{index}]");
            sleep(Duration::from_millis(100));
        }
    });

    pool.execute(|| {
        for index in 0..100 {
            println!("In Task2 [{index}]");
            sleep(Duration::from_millis(100));
        }
    });

    pool.execute(|| {
        for index in 0..100 {
            println!("In Task3 [{index}]");
            sleep(Duration::from_millis(100));
        }
    });
}
