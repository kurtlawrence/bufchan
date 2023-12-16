use parking_lot::Mutex;
use std::{collections::VecDeque, sync::Arc};

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    unbounded_with_buffer(512)
}

pub fn unbounded_with_buffer<T>(buf_size: usize) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Mutex::new(Default::default()));
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
// ###### SENDER ###############################################################

pub struct Sender<T> {
    local: Vec<T>,
    shared: Arc<Mutex<VecDeque<T>>>,
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
        if let Some(mut q) = self.shared.try_lock() {
            // if we manage to get a lock, extend the queue with our buffered items
            q.extend(self.local.drain(..));
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.local.is_empty() {
            return; // no buffered items, can exit
        }
        // we have to block until we get the q
        self.shared.lock().extend(self.local.drain(..));
    }
}

// ###### RECEIVER #############################################################

pub struct Receiver<T> {
    local: VecDeque<T>,
    shared: Arc<Mutex<VecDeque<T>>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        // drain from the buffer first
        if let Some(x) = self.local.pop_front() {
            return Some(x);
        }
        // else wait until we can get a lock and take from the shared queue
        loop {
            let ref_count = Arc::strong_count(&self.shared);
            if let Some(mut q) = self
                .shared
                .try_lock_for(std::time::Duration::from_millis(10))
            {
                // since our buffer is empty, we can just swap it out
                std::mem::swap(&mut self.local, &mut q);
                match self.local.pop_front() {
                    // buffer has been filled again, return it
                    Some(x) => return Some(x),
                    None => {
                        // now this is interesting!
                        // there are no more items in the buffer or shared queue
                        // check the ref count, if only one then there will be no more items!
                        if ref_count == 1 {
                            return None; // all finished
                        }
                    }
                }
            }
        }
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}
