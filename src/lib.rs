use std::{
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
    pub fn new(thread_num: usize) -> Self {
        assert!(thread_num > 0);

        let queue = Arc::new(BlockingQueue::new());

        let mut workers = Vec::with_capacity(thread_num);
        for id in 0..thread_num {
            workers.push(Worker::new(id, queue.clone()));
        }

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
