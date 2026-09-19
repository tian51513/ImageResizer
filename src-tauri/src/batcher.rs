// Coalesces per-file progress events into time-sliced batches, so a
// 10k-file run fires ~hundreds of IPC events instead of 10k.

use crate::config::{ProcessResult, ProgressBatch, ProgressEvent};
use std::time::{Duration, Instant};

pub struct ProgressBatcher {
    pending: Vec<ProcessResult>,
    last_event: Option<ProgressEvent>,
    interval: Duration,
    last_emit: Instant,
}

impl ProgressBatcher {
    pub fn new(interval: Duration) -> Self {
        ProgressBatcher {
            pending: Vec::new(),
            last_event: None,
            interval,
            last_emit: Instant::now(),
        }
    }

    /// Record a finished file; returns a batch when the emit interval
    /// has elapsed, otherwise holds the row for the next batch.
    pub fn record(&mut self, event: ProgressEvent) -> Option<ProgressBatch> {
        self.pending.push(ProcessResult {
            file: event.file.clone(),
            original_size: event.original_size,
            new_size: event.new_size,
            status: event.status.clone(),
        });
        self.last_event = Some(event);
        if self.last_emit.elapsed() >= self.interval {
            self.take()
        } else {
            None
        }
    }

    /// Emit whatever is pending, regardless of the interval (end of batch).
    pub fn flush(&mut self) -> Option<ProgressBatch> {
        self.take()
    }

    fn take(&mut self) -> Option<ProgressBatch> {
        if self.pending.is_empty() {
            return None;
        }
        self.last_emit = Instant::now();
        Some(ProgressBatch {
            last: self.last_event.clone()?,
            results: std::mem::take(&mut self.pending),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ProcessResult, ProgressEvent};
    use std::time::Duration;

    fn ev(current: u32, file: &str) -> ProgressEvent {
        ProgressEvent {
            total: 10,
            current,
            file: file.to_string(),
            original_size: 100,
            new_size: 50,
            status: "success".to_string(),
            total_original_bytes: 1000,
            processed_bytes: current as u64 * 100,
        }
    }

    #[test]
    fn holds_back_events_before_interval_elapses() {
        let mut b = ProgressBatcher::new(Duration::from_secs(60));
        assert!(b.record(ev(1, "a.jpg")).is_none(), "not due yet");
        assert!(b.record(ev(2, "b.jpg")).is_none(), "still not due");
    }

    #[test]
    fn emits_batch_once_interval_elapsed() {
        let mut b = ProgressBatcher::new(Duration::from_millis(0)); // always due
        let batch = b.record(ev(1, "a.jpg")).expect("zero interval must emit");
        assert_eq!(batch.results.len(), 1);
        assert_eq!(batch.results[0].file, "a.jpg");
        assert_eq!(batch.last.current, 1, "batch carries the latest counters");
    }

    #[test]
    fn flush_returns_pending_and_empties() {
        let mut b = ProgressBatcher::new(Duration::from_secs(60));
        b.record(ev(1, "a.jpg"));
        b.record(ev(2, "b.jpg"));
        let batch = b.flush().expect("pending events must flush");
        assert_eq!(batch.results.len(), 2);
        assert_eq!(batch.last.current, 2);
        assert!(b.flush().is_none(), "second flush has nothing");
    }

    #[test]
    fn emitted_batch_clears_the_accumulator() {
        let mut b = ProgressBatcher::new(Duration::from_secs(60));
        b.record(ev(1, "a.jpg"));
        b.record(ev(2, "b.jpg"));
        let batch = b.flush().unwrap();
        assert_eq!(batch.results.len(), 2);
        // after flush, the next interval's batch starts fresh
        b.record(ev(3, "c.jpg"));
        let next = b.flush().unwrap();
        assert_eq!(next.results.len(), 1);
        assert_eq!(next.results[0].file, "c.jpg");
    }

    #[test]
    fn pending_results_map_from_events() {
        let mut b = ProgressBatcher::new(Duration::from_secs(60));
        b.record(ev(1, "a.jpg"));
        let batch = b.flush().unwrap();
        let expected = ProcessResult {
            file: "a.jpg".to_string(),
            original_size: 100,
            new_size: 50,
            status: "success".to_string(),
        };
        assert_eq!(batch.results[0].file, expected.file);
        assert_eq!(batch.results[0].original_size, expected.original_size);
        assert_eq!(batch.results[0].new_size, expected.new_size);
        assert_eq!(batch.results[0].status, expected.status);
    }
}
