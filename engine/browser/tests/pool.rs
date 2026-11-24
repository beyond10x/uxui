//! The worker pool a checks run measures its pages with: items in order, each once, at most `jobs`
//! workers at a time, each with a state of its own, and the first failure stopping the run.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use ux_browser::pool;

#[test]
fn every_item_runs_once_and_the_results_come_back_in_item_order() {
    let runs = Mutex::new(Vec::new());
    let inits = AtomicUsize::new(0);
    let finished = AtomicUsize::new(0);
    let out = pool::run(
        3,
        10,
        || Ok(inits.fetch_add(1, Ordering::SeqCst)),
        |_worker: &mut usize, i| {
            runs.lock().unwrap().push(i);
            Ok(i * 10)
        },
        |_worker| {
            finished.fetch_add(1, Ordering::SeqCst);
        },
    )
    .unwrap();
    assert_eq!(out, (0..10).map(|i| i * 10).collect::<Vec<_>>());
    let mut seen = runs.into_inner().unwrap();
    seen.sort();
    assert_eq!(seen, (0..10).collect::<Vec<_>>());
    let inits = inits.load(Ordering::SeqCst);
    assert!((1..=3).contains(&inits), "{inits} workers");
    assert_eq!(
        finished.load(Ordering::SeqCst),
        inits,
        "every worker is finished"
    );
}

#[test]
fn no_more_workers_start_than_there_are_items() {
    let inits = AtomicUsize::new(0);
    let out = pool::run(
        4,
        2,
        || Ok(inits.fetch_add(1, Ordering::SeqCst)),
        |_w: &mut usize, i| Ok(i),
        |_w| {},
    )
    .unwrap();
    assert_eq!(out, [0, 1]);
    assert!(inits.load(Ordering::SeqCst) <= 2);
    let none: Vec<usize> = pool::run(4, 0, || Ok(()), |_w: &mut (), i| Ok(i), |_w| {}).unwrap();
    assert!(none.is_empty());
}

#[test]
fn the_workers_run_at_the_same_time() {
    let active = AtomicUsize::new(0);
    let most = AtomicUsize::new(0);
    pool::run(
        4,
        4,
        || Ok(()),
        |_w: &mut (), _i| {
            let now = active.fetch_add(1, Ordering::SeqCst) + 1;
            most.fetch_max(now, Ordering::SeqCst);
            // Each item waits until all four are in, or gives up: a pool that ran them one after
            // another ends with a high-water mark of 1.
            let deadline = Instant::now() + Duration::from_secs(5);
            while most.load(Ordering::SeqCst) < 4 && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(1));
            }
            active.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        },
        |_w| {},
    )
    .unwrap();
    assert_eq!(most.load(Ordering::SeqCst), 4);
}

#[test]
fn a_failure_stops_the_run_and_the_first_failed_item_names_it() {
    let finished = AtomicUsize::new(0);
    let inits = AtomicUsize::new(0);
    let r: Result<Vec<usize>, String> = pool::run(
        2,
        50,
        || Ok(inits.fetch_add(1, Ordering::SeqCst)),
        |_w: &mut usize, i| {
            if i >= 5 {
                Err(format!("item {i} failed"))
            } else {
                Ok(i)
            }
        },
        |_w| {
            finished.fetch_add(1, Ordering::SeqCst);
        },
    );
    assert_eq!(r.unwrap_err(), "item 5 failed");
    assert_eq!(
        finished.load(Ordering::SeqCst),
        inits.load(Ordering::SeqCst),
        "a stopped run still finishes its workers"
    );
}

#[test]
fn a_worker_that_does_not_start_fails_the_run() {
    let r: Result<Vec<usize>, String> = pool::run(
        2,
        3,
        || Err::<(), String>("no connection".to_string()),
        |_w: &mut (), i| Ok(i),
        |_w| {},
    );
    assert_eq!(r.unwrap_err(), "no connection");
}

#[test]
fn the_default_number_of_jobs_is_the_cpus_up_to_four() {
    assert_eq!(pool::default_jobs(1), 1);
    assert_eq!(pool::default_jobs(2), 2);
    assert_eq!(pool::default_jobs(4), 4);
    assert_eq!(pool::default_jobs(20), 4);
    assert_eq!(pool::default_jobs(0), 1);
}

fn vp(name: &str) -> uxui_doc::yaml::Value {
    ux_browser::json::parse(&format!(r#"{{"viewport":"{name}"}}"#)).unwrap()
}

#[test]
fn a_page_is_its_viewports_or_the_first_failure_in_viewport_order() {
    use ux_browser::measure::{Failure, page_record};
    let whole = page_record(
        "/llm/",
        vec![
            ("1440x900".to_string(), Ok(vp("1440x900"))),
            ("390x844".to_string(), Ok(vp("390x844"))),
        ],
    )
    .unwrap();
    assert_eq!(whole.get_str("path"), Some("/llm/"));
    assert_eq!(whole.get("viewports").unwrap().list().len(), 2);
    assert_eq!(whole.get("error"), None);

    let second = page_record(
        "/llm/x",
        vec![
            ("1440x900".to_string(), Ok(vp("1440x900"))),
            (
                "390x844".to_string(),
                Err(Failure::Page("HTTP status 404".to_string())),
            ),
        ],
    )
    .unwrap();
    assert_eq!(second.get_str("error"), Some("HTTP status 404"));
    assert_eq!(second.get("viewports"), None);

    let first = page_record(
        "/llm/x",
        vec![
            (
                "1440x900".to_string(),
                Err(Failure::Page("HTTP status 500".to_string())),
            ),
            (
                "390x844".to_string(),
                Err(Failure::Browser("gone".to_string())),
            ),
        ],
    )
    .unwrap();
    assert_eq!(first.get_str("error"), Some("HTTP status 500"));

    let broken = page_record(
        "/llm/x",
        vec![
            (
                "1440x900".to_string(),
                Err(Failure::Browser("no answer".to_string())),
            ),
            ("390x844".to_string(), Ok(vp("390x844"))),
        ],
    );
    assert_eq!(broken.unwrap_err(), "/llm/x 1440x900: no answer");
}
