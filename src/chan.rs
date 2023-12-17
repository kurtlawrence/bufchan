use parking_lot::{Condvar, Mutex};
use std::{collections::VecDeque, sync::Arc};

/// Create an unbounded channel with default buffer size.
pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    unbounded_with_buffer(512)
}

/// Create an unbounded channel with the specified buffer size.
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

/// The sending side of the channel.
///
/// `Sender` is `Send`, `Sync`, and `Clone`able.
///
/// When `Sender` is dropped, any locally buffered messages are flushed to the receiver.
/// Note that this _might_ block if waiting on the shared lock.
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
    /// Buffer a message to be sent.
    /// If the local buffer reaches the threshold, an attempt to send the buffered messages to the
    /// receiver occurs.
    ///
    /// This is guaranteed not to block.
    pub fn send(&mut self, val: T) {
        // push onto buffer
        self.local.push(val);
        if self.local.len() < self.lim {
            return; // haven't reached threshold
        }

        let (lock, cvar) = self.shared.as_ref();
        if let Some(mut q) = lock.try_lock() {
            // if we manage to get a lock, extend the queue with our buffered items
            q.extend(self.local.drain(..));
            // notify that the queue has been updated
            cvar.notify_one();
        }
    }

    /// Flush any locally buffered items to the receiver.
    ///
    /// This is blocking.
    pub fn flush(&mut self) {
        if self.local.is_empty() {
            return; // no buffered items, can exit
        }
        let (lock, cvar) = self.shared.as_ref();
        // we have to block until we get the q
        lock.lock().extend(self.local.drain(..));
        // notify that the queue has been updated
        cvar.notify_one();
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.flush();
        self.shared.1.notify_one();
    }
}

// ###### RECEIVER #############################################################

/// The receiving end of the channel.
///
/// There can be only _one_ receiver.
pub struct Receiver<T> {
    local: VecDeque<T>,
    shared: Shared<T>,
}

impl<T> Receiver<T> {
    /// Receive a message, return `None` is there are no more messages _or senders_.
    ///
    /// Receive will block until a message becomes available or all associated [`Sender`]s are
    /// dropped.
    pub fn recv(&mut self) -> Option<T> {
        // only check for messages if local buffer is empty
        while self.local.is_empty() {
            if Arc::strong_count(&self.shared) == 1 {
                // there are no more senders, take the shared queue and break
                // if both queues are empty, breaking will return None and signals
                // end of channel
                std::mem::swap(&mut self.local, &mut self.shared.0.lock());
                break;
            } else {
                let (lock, cvar) = self.shared.as_ref();
                // obtain a lock on the shared queue
                let mut q = lock.lock();
                if q.is_empty() {
                    // if queue is empty, unlock it and wait for a sender to notify
                    // that it has been updated
                    cvar.wait_for(&mut q, std::time::Duration::from_millis(100));
                }
                // our local buffer is empty, so we can swap the shared one for it
                std::mem::swap(&mut self.local, &mut q);
            }
        }
        // pull from the local buffer
        self.local.pop_front()
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}
