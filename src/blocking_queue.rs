use std::{
    collections::LinkedList,
    sync::{Condvar, Mutex},
};

pub struct BlockingQueue<T> {
    list: Mutex<LinkedList<T>>,
    cvar: Condvar,
}

impl<T> BlockingQueue<T> {
    pub const fn new() -> Self {
        Self {
            list: Mutex::new(LinkedList::new()),
            cvar: Condvar::new(),
        }
    }

    pub fn push(&self, data: T) {
        self.list.lock().unwrap().push_back(data);
        self.cvar.notify_one();
    }

    pub fn pop(&self) -> T {
        let mut list = self.list.lock().unwrap();

        while list.is_empty() {
            list = self.cvar.wait(list).unwrap();
        }

        list.pop_front().unwrap()
    }
}

impl<T> Default for BlockingQueue<T> {
    fn default() -> Self {
        const { Self::new() }
    }
}
