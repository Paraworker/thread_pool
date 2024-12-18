use std::{
    num::NonZeroUsize,
    sync::Arc,
    thread::{self, JoinHandle},
};

use blocking_queue::BlockingQueue;
use log::info;

mod blocking_queue;

enum Message {
    NewTask(Box<dyn FnOnce() + Send>),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    queue: Arc<BlockingQueue<Message>>,
}

impl ThreadPool {
    /// Create a thread pool with the given thread number
    pub fn new(thread_num: NonZeroUsize) -> Self {
        let queue = Arc::new(BlockingQueue::new());

        let workers = (0..thread_num.get())
            .map(|id| Worker::new(id, queue.clone()))
            .collect();

        Self { workers, queue }
    }

    /// Execute a task
    pub fn execute<Task>(&self, task: Task)
    where
        Task: FnOnce() + Send + 'static,
    {
        self.queue.push(Message::NewTask(Box::new(task)));
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            self.queue.push(Message::Terminate);
        }

        self.workers.clear();
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, queue: Arc<BlockingQueue<Message>>) -> Self {
        let handle = thread::spawn(move || loop {
            match queue.pop() {
                Message::NewTask(task) => {
                    info!("Worker[{id}] received a task!");
                    task();
                }
                Message::Terminate => {
                    info!("Worker[{id}] terminate!");
                    break;
                }
            }
        });

        Self {
            thread: Some(handle),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.thread.take().unwrap().join().unwrap();
    }
}
