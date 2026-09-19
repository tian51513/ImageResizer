// Weighted memory gate: caps total in-flight estimated image memory.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};

/// Weighted counting semaphore over a byte budget. Caps how much estimated
/// image memory may be processed concurrently, so `memory_budget_mb` is a
/// real limit instead of a hint. A request larger than the whole budget is
/// capped at the budget so it can still run alone (progress over deadlock).
pub struct MemoryGate {
    budget: u64,
    in_use: AtomicU64,
    waiters_lock: Mutex<()>,
    cv: Condvar,
}

/// RAII grant returned by [`MemoryGate::acquire`]; releasing the bytes
/// happens on drop, which wakes blocked waiters.
pub struct MemoryLease<'a> {
    gate: &'a MemoryGate,
    cost: u64,
}

impl MemoryGate {
    pub fn new(budget_bytes: u64) -> Self {
        Self {
            budget: budget_bytes.max(1),
            in_use: AtomicU64::new(0),
            waiters_lock: Mutex::new(()),
            cv: Condvar::new(),
        }
    }

    /// Block until `requested` bytes fit within the budget, then grant them.
    pub fn acquire(&self, requested: u64) -> MemoryLease<'_> {
        let cost = requested.min(self.budget);
        let mut guard = self.waiters_lock.lock().unwrap();
        loop {
            let free = self
                .budget
                .saturating_sub(self.in_use.load(Ordering::Relaxed));
            if free >= cost {
                self.in_use.fetch_add(cost, Ordering::Relaxed);
                return MemoryLease { gate: self, cost };
            }
            guard = self.cv.wait(guard).unwrap();
        }
    }

    fn release(&self, cost: u64) {
        self.in_use.fetch_sub(cost, Ordering::Relaxed);
        // Take the lock before notifying so a waiter cannot miss the wakeup
        // between checking the condition and starting to wait.
        let _guard = self.waiters_lock.lock().unwrap();
        self.cv.notify_all();
    }
}

impl Drop for MemoryLease<'_> {
    fn drop(&mut self) {
        self.gate.release(self.cost);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn grants_within_budget_without_blocking() {
        let gate = MemoryGate::new(100);
        let a = gate.acquire(30);
        let _b = gate.acquire(30);
        drop(a);
    }

    #[test]
    fn oversized_request_grants_alone_without_deadlock() {
        let gate = MemoryGate::new(100);
        let _lease = gate.acquire(5000); // must not hang
    }

    #[test]
    fn concurrent_grants_never_exceed_budget() {
        let gate = Arc::new(MemoryGate::new(100));
        let peak = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];
        for _ in 0..4 {
            let gate = Arc::clone(&gate);
            let peak = Arc::clone(&peak);
            let active = Arc::clone(&active);
            handles.push(std::thread::spawn(move || {
                let _lease = gate.acquire(40);
                let cur = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(cur, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(80));
                active.fetch_sub(1, Ordering::SeqCst);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let peak = peak.load(Ordering::SeqCst);
        assert!(
            peak <= 2,
            "budget 100 must cap concurrent 40-byte grants at 2, peak was {}",
            peak
        );
    }

    #[test]
    fn dropping_lease_frees_capacity_for_blocked_waiter() {
        let gate = Arc::new(MemoryGate::new(100));
        let lease = gate.acquire(60);
        let gate2 = Arc::clone(&gate);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _l = gate2.acquire(60);
            tx.send(()).unwrap();
            std::thread::sleep(Duration::from_millis(30));
        });
        std::thread::sleep(Duration::from_millis(50));
        drop(lease); // must wake the blocked waiter
        rx.recv_timeout(Duration::from_secs(3))
            .expect("waiter must acquire after lease dropped");
    }
}
