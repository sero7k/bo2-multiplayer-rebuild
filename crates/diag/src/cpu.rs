//! The calling thread's CPU time: whether a slow stretch was work, or the
//! thread waiting (on the disk, a lock, or other programs on the PC).

/// The calling thread's CPU time so far (ms), where it can be read.
pub fn thread_cpu_ms() -> Option<f64> {
    imp::thread_cpu_ms()
}

/// The whole program's CPU time so far (ms, all its threads), where it can
/// be read: a long frame that took little of it was the program waiting.
pub fn process_cpu_ms() -> Option<f64> {
    imp::process_cpu_ms()
}

/// For a slow stretch's log line: how much of it the thread spent working,
/// from its CPU time at the start (`thread_cpu_ms`); empty where not read.
pub fn busy_note(at_start: Option<f64>) -> String {
    match (at_start, thread_cpu_ms()) {
        (Some(a), Some(b)) => format!(", working {:.1} ms of it", (b - a).max(0.0)),
        _ => String::new(),
    }
}

#[cfg(all(windows, target_arch = "x86_64"))]
mod imp {
    use std::sync::OnceLock;
    use std::time::Instant;

    unsafe extern "system" {
        fn GetCurrentThread() -> isize;
        fn QueryThreadCycleTime(thread: isize, cycles: *mut u64) -> i32;
        fn GetCurrentProcess() -> isize;
        fn GetProcessTimes(process: isize, created: *mut u64, exited: *mut u64, kernel: *mut u64, user: *mut u64) -> i32;
    }

    pub(super) fn process_cpu_ms() -> Option<f64> {
        let (mut created, mut exited, mut kernel, mut user) = (0u64, 0u64, 0u64, 0u64);
        // SAFETY: the current process's pseudo handle, and four FILETIMEs
        // (100 ns counts, 8-byte aligned here) to write.
        let ok = unsafe {
            GetProcessTimes(GetCurrentProcess(), &raw mut created, &raw mut exited, &raw mut kernel, &raw mut user)
        };
        (ok != 0).then(|| (kernel + user) as f64 / 10_000.0)
    }

    /// The first reading's clock and time-stamp counter: the counter's rate
    /// (the thread's cycles are counted in it) from how far both have gone.
    static START: OnceLock<(Instant, u64)> = OnceLock::new();

    pub(super) fn thread_cpu_ms() -> Option<f64> {
        let mut cycles = 0u64;
        // SAFETY: the current thread's pseudo handle, and a u64 to write.
        if unsafe { QueryThreadCycleTime(GetCurrentThread(), &raw mut cycles) } == 0 {
            return None;
        }
        // SAFETY: reads the time-stamp counter (every x86_64 has it).
        let tsc = unsafe { core::arch::x86_64::_rdtsc() };
        let (t0, c0) = *START.get_or_init(|| (Instant::now(), tsc));
        let secs = t0.elapsed().as_secs_f64();
        if secs < 0.25 {
            return None;
        }
        let hz = tsc.wrapping_sub(c0) as f64 / secs;
        (hz > 0.0).then(|| cycles as f64 / hz * 1000.0)
    }
}

#[cfg(not(all(windows, target_arch = "x86_64")))]
mod imp {
    pub(super) fn thread_cpu_ms() -> Option<f64> {
        None
    }

    pub(super) fn process_cpu_ms() -> Option<f64> {
        None
    }
}
