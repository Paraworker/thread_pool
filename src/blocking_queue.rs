use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

pub struct BlockingQueue<T> {
    deque: Mutex<VecDeque<T>>,
    cvar: Condvar,
}

impl<T> BlockingQueue<T> {
    pub const fn new() -> Self {
        Self {
            deque: Mutex::new(VecDeque::new()),
            cvar: Condvar::new(),
        }
    }

    pub fn push(&self, data: T) {
        self.deque.lock().unwrap().push_back(data);
        self.cvar.notify_one();
    }

    pub fn pop(&self) -> T {
        let mut deque = self.deque.lock().unwrap();

        while deque.is_empty() {
            deque = self.cvar.wait(deque).unwrap();
        }

        deque.pop_front().unwrap()
    }
}

impl<T> Default for BlockingQueue<T> {
    fn default() -> Self {
        const { Self::new() }
    }
}
