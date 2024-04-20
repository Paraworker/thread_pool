use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    sender: Sender<Task>,
    workers: Vec<Worker>,
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

        Self { sender, workers }
    }

    /// Push a task
    pub fn push(&self, task: impl FnOnce() + Send + 'static) {
        self.sender.send(Box::new(task)).unwrap();
    }
}

struct Worker(Option<JoinHandle<()>>);

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Task>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                let task = receiver
                    .lock()
                    .unwrap()
                    .recv();

                match task {
                    Ok(task) => {
                        println!("Worker[{id}] received a task!");
                        task();
                    },
                    Err(_) => {
                        println!("Worker[{id}] disconnected!");
                        break;
                    },
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

    use std::{thread::sleep, time::Duration};

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
