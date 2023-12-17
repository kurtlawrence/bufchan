use parking_lot::{Condvar, Mutex};
use std::{collections::VecDeque, sync::Arc};

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    unbounded_with_buffer(512)
}

pub fn unbounded_with_buffer<T>(buf_size: usize) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new((Mutex::new(Default::default()), Condvar::new()));
    (
        Sender {
            local: Default::default(),
            shared: Arc::clone(&shared),
            lim: buf_size,
        },
        Receiver {
            local: Default::default(),
            shared,
        },
    )
}

type Shared<T> = Arc<(Mutex<VecDeque<T>>, Condvar)>;

// ###### SENDER ###############################################################

pub struct Sender<T> {
    local: Vec<T>,
    shared: Shared<T>,
    lim: usize,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            local: Default::default(),
            shared: Arc::clone(&self.shared),
            lim: self.lim,
        }
    }
}

impl<T> Sender<T> {
    pub fn send(&mut self, val: T) {
        // push onto buffer
        self.local.push(val);
        if self.local.len() < self.lim {
            return; // haven't reached threshold
        }

        let &(ref lock, ref cvar) = self.shared.as_ref();
        if let Some(mut q) = lock.try_lock() {
            // if we manage to get a lock, extend the queue with our buffered items
            q.extend(self.local.drain(..));
            // notify that the queue has been updated
            cvar.notify_one();
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        /*
        if self.local.is_empty() {
            return; // no buffered items, can exit
        }
        */
        // we have to block until we get the q
        let &(ref lock, ref cvar) = self.shared.as_ref();
        let mut q = lock.lock();
        q.extend(self.local.drain(..));
        cvar.notify_one();
    }
}

// ###### RECEIVER #############################################################

pub struct Receiver<T> {
    local: VecDeque<T>,
    shared: Shared<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        while self.local.is_empty() {
            if Arc::strong_count(&self.shared) == 1 {
                std::mem::swap(&mut self.local, &mut self.shared.0.lock());
                break;
            } else {
                let &(ref lock, ref cvar) = self.shared.as_ref();
                let mut q = lock.lock();
                if q.is_empty() {
                    cvar.wait_for(&mut q, std::time::Duration::from_millis(100));
                }
                std::mem::swap(&mut self.local, &mut q);
            }
        }
        self.local.pop_front()
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}
