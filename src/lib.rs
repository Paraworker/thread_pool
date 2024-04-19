use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

type Task = Box<dyn FnOnce() + Send + 'static>;
type ThreadData = Arc<(AtomicBool, Mutex<Receiver<Task>>)>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Task>,
    thread_data: ThreadData,
}

impl ThreadPool {
    /// Create a thread pool with the given thread number
    pub fn new(thread_num: usize) -> Self {
        assert!(thread_num > 0);

        let (sender, receiver) = channel();
        let thread_data = Arc::new((AtomicBool::new(true), Mutex::new(receiver)));

        let mut workers = Vec::with_capacity(thread_num);
        for id in 0..thread_num {
            workers.push(Worker::new(id, thread_data.clone()));
        }

        Self { workers, sender, thread_data }
    }

    /// Push a task
    pub fn push(&self, task: impl FnOnce() + Send + 'static) {
        self.sender.send(Box::new(task)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.thread_data.0.store(false, Ordering::Release);
    }
}

struct Worker(Option<JoinHandle<()>>);

impl Worker {
    fn new(id: usize, thread_data: ThreadData) -> Self {
        let handle = thread::spawn(move || {
            while thread_data.0.load(Ordering::Acquire) {
                let task = thread_data.1
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_millis(1000));

                if let Ok(task) = task {
                    println!("Worker[{id}] received a task!");
                    task();
                }
            }
        });

        Self(Some(handle))
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.0
            .take()
            .unwrap()
            .join()
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::thread::sleep;

    #[test]
    fn push_tasks() {
        let pool = ThreadPool::new(5);

        sleep(Duration::from_millis(1000));

        pool.push(|| {
            for index in 0..100 {
                println!("In Task1 [{index}]");
                sleep(Duration::from_millis(100));
            }
        });

        pool.push(|| {
            for index in 0..100 {
                println!("In Task2 [{index}]");
                sleep(Duration::from_millis(100));
            }
        });

        pool.push(|| {
            for index in 0..100 {
                println!("In Task3 [{index}]");
                sleep(Duration::from_millis(100));
            }
        });
    }
}
