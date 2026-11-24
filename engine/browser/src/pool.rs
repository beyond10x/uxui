//! A small pool of worker threads: items `0..count` are taken in order by up to `jobs` workers, each
//! with a state of its own (a checks run: one DevTools connection and its pages). The first failure
//! stops the others from taking more items.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// The workers a run starts when it is not told: the CPUs, at most 4, at least 1.
pub fn default_jobs(cpus: usize) -> usize {
    cpus.clamp(1, 4)
}

/// The CPUs this process may use.
pub fn cpus() -> usize {
    std::thread::available_parallelism().map_or(1, usize::from)
}

/// Runs `work` on every item with up to `jobs` workers. A worker's state comes from `init` and is
/// handed to `finish` when the worker ends, also after a failure. Returns the results in item
/// order, or the failure of the lowest failed item (or of a worker that did not start).
pub fn run<S, T, I, W, F>(
    jobs: usize,
    count: usize,
    init: I,
    work: W,
    finish: F,
) -> Result<Vec<T>, String>
where
    T: Send,
    I: Fn() -> Result<S, String> + Sync,
    W: Fn(&mut S, usize) -> Result<T, String> + Sync,
    F: Fn(S) + Sync,
{
    let workers = jobs.max(1).min(count);
    let next = AtomicUsize::new(0);
    let stop = AtomicBool::new(false);
    let slots: Vec<Mutex<Option<Result<T, String>>>> =
        (0..count).map(|_| Mutex::new(None)).collect();
    let start_failure: Mutex<Option<String>> = Mutex::new(None);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                let mut state = match init() {
                    Ok(s) => s,
                    Err(e) => {
                        stop.store(true, Ordering::SeqCst);
                        start_failure.lock().unwrap().get_or_insert(e);
                        return;
                    }
                };
                while !stop.load(Ordering::SeqCst) {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= count {
                        break;
                    }
                    let result = work(&mut state, i);
                    if result.is_err() {
                        stop.store(true, Ordering::SeqCst);
                    }
                    *slots[i].lock().unwrap() = Some(result);
                }
                finish(state);
            });
        }
    });
    if let Some(e) = start_failure.into_inner().unwrap() {
        return Err(e);
    }
    let mut out = Vec::with_capacity(count);
    for slot in slots {
        match slot.into_inner().unwrap() {
            Some(Ok(v)) => out.push(v),
            Some(Err(e)) => return Err(e),
            None => return Err("an item was not run".to_string()),
        }
    }
    Ok(out)
}
