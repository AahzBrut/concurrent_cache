use parking_lot::{Condvar, Mutex};

pub struct Semaphore {
    counter: Mutex<usize>,
    condvar: Condvar,
    limit: usize,
}

pub struct SemaphoreGuard<'a> {
    semaphore: &'a Semaphore,
}

impl Semaphore {
    pub fn new(limit: usize) -> Self {
        Self {
            counter: Mutex::new(0),
            condvar: Condvar::new(),
            limit,
        }
    }

    pub fn acquire(&self) -> SemaphoreGuard {
        let mut counter = self.counter.lock();
        while *counter >= self.limit {
            self.condvar.wait(&mut counter);
        }
        *counter += 1;
        SemaphoreGuard { semaphore: self }
    }

    fn decrement(&self) {
        let mut counter = self.counter.lock();
        *counter -= 1;
        self.condvar.notify_one();
    }
}

impl Drop for SemaphoreGuard<'_> {
    fn drop(&mut self) {
        self.semaphore.decrement();
    }
}
