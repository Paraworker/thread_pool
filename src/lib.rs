use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;
use std::{
    num::NonZeroUsize,
    thread::{self, JoinHandle},
};

type Task = Box<dyn FnOnce() + Send>;

pub struct ThreadPool {
    /// Order matters here
    /// Drop sender first to stop workers
    sender: Sender<Task>,

    #[allow(dead_code)]
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Create a thread pool with the given thread number
    pub fn new(thread_num: NonZeroUsize) -> Self {
        let (sender, receiver) = unbounded();

        let workers = (0..thread_num.get())
            .map(|id| Worker::spawn(id, receiver.clone()))
            .collect();

        Self { sender, workers }
    }

    /// Execute a task
    pub fn execute<Task>(&self, task: Task)
    where
        Task: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(task)).unwrap();
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn spawn(id: usize, receiver: Receiver<Task>) -> Self {
        Self {
            thread: Some(thread::spawn(move || Self::task_loop(id, receiver))),
        }
    }

    fn task_loop(id: usize, receiver: Receiver<Task>) {
        while let Ok(task) = receiver.recv() {
            info!("Worker[{id}] received a task!");
            task();
        }

        info!("Worker[{id}] terminate!");
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.thread.take().unwrap().join().unwrap();
    }
}
