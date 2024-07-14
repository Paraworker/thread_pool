use std::{collections::LinkedList, sync::{Condvar, Mutex}};

pub(crate) struct BlockingQueue<T> {
    list: Mutex<LinkedList<T>>,
    cvar: Condvar,
}

impl<T> BlockingQueue<T> {
    pub(crate) fn new() -> Self {
        Self {
            list: Mutex::new(LinkedList::new()),
            cvar: Condvar::new(),
        }
    }

    pub(crate) fn push(&self, data: T) {
        self.list
            .lock()
            .unwrap()
            .push_back(data);

        self.cvar.notify_one();
    }

    pub(crate) fn pop(&self) -> T {
        let mut guard = self.list
            .lock()
            .unwrap();

        while guard.is_empty() {
            guard = self.cvar
                .wait(guard)
                .unwrap();
        }

        guard.pop_front().unwrap()
    }
}
