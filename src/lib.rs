use std::{
    sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex},
    thread::{self, JoinHandle},
};
use log::info;

enum Message {
    NewTask(Box<dyn FnOnce() + Send + 'static>),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
}

impl ThreadPool {
    /// Create a thread pool with the given thread number
    pub fn new(thread_num: usize) -> Self {
        assert!(thread_num > 0);

        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(thread_num);
        for id in 0..thread_num {
            workers.push(Worker::new(id, receiver.clone()));
        }

        Self { workers, sender }
    }

    /// Execute a task
    pub fn execute<Task>(&self, task: Task)
    where
        Task: FnOnce() + Send + 'static
    {
        self.sender.send(Message::NewTask(Box::new(task))).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            self.sender.send(Message::Terminate).unwrap();
        }

        self.workers.clear();
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                let message = receiver
                    .lock()
                    .unwrap()
                    .recv()
                    .unwrap();

                match message {
                    Message::NewTask(task) => {
                        info!("Worker[{id}] received a task!");
                        task();
                    },
                    Message::Terminate => {
                        info!("Worker[{id}] terminate!");
                        break;
                    },
                }
            }
        });

        Self { thread: Some(handle) }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.thread
            .take()
            .unwrap()
            .join()
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{thread::sleep, time::Duration};

    #[test]
    fn execute_tasks() {
        let pool = ThreadPool::new(5);

        sleep(Duration::from_millis(1000));

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
}
