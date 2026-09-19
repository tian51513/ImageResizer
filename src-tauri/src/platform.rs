// Platform integration: background-friendly scheduling knobs.

/// Number of worker threads for batch processing: all cores minus one,
/// so the OS and foreground apps always keep a core to themselves.
pub fn worker_thread_count() -> usize {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    cores.saturating_sub(1).max(1)
}

/// Drop the calling thread's priority to below normal: the batch still uses
/// full CPU while the machine is idle, but yields instantly whenever a
/// foreground app needs the core. No-op on non-Windows platforms.
#[cfg(windows)]
pub fn lower_thread_priority() {
    unsafe {
        windows_sys::Win32::System::Threading::SetThreadPriority(
            windows_sys::Win32::System::Threading::GetCurrentThread(),
            windows_sys::Win32::System::Threading::THREAD_PRIORITY_BELOW_NORMAL,
        );
    }
}

#[cfg(not(windows))]
pub fn lower_thread_priority() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_thread_count_leaves_headroom() {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let n = worker_thread_count();
        assert!(n >= 1, "at least one worker");
        assert!(n <= cores, "never exceed available cores");
        if cores > 1 {
            assert!(n < cores, "must reserve at least one core for the system/foreground");
        }
    }

    #[cfg(windows)]
    #[test]
    fn lower_thread_priority_actually_lowers_on_windows() {
        // Priority starts normal (0) on a fresh test thread; after the call it
        // must read back as below normal (-1).
        unsafe {
            let before = windows_sys::Win32::System::Threading::GetThreadPriority(
                windows_sys::Win32::System::Threading::GetCurrentThread(),
            );
            assert_eq!(before, 0, "test precondition: priority should start normal");
            lower_thread_priority();
            let after = windows_sys::Win32::System::Threading::GetThreadPriority(
                windows_sys::Win32::System::Threading::GetCurrentThread(),
            );
            assert_eq!(
                after,
                windows_sys::Win32::System::Threading::THREAD_PRIORITY_BELOW_NORMAL,
                "priority must be below normal after the call"
            );
        }
    }
}
