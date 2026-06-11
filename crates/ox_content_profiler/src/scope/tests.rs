use std::sync::Mutex;

use super::*;

/// `enable()` / `disable()` write to a process-wide `AtomicBool`, but
/// `cargo test` runs unit tests in parallel by default. Without this
/// lock the three tests below would race: e.g. `disabled_spans_are_noops`
/// could flip the gate off while `span_records_self_and_inclusive_time`
/// was mid-`span("outer", ...)`, producing an empty records list and
/// a flaky "outer recorded" panic. Serialize them through one mutex so
/// they observe the gate state they set themselves.
static GATE: Mutex<()> = Mutex::new(());

#[test]
fn span_records_self_and_inclusive_time() {
    let _guard = GATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    enable();
    reset_thread_spans();

    span("outer", || {
        std::thread::sleep(Duration::from_millis(2));
        span("inner", || {
            std::thread::sleep(Duration::from_millis(5));
        });
    });

    let records = take_thread_spans();
    let outer = records.iter().find(|r| r.name == "outer").expect("outer recorded");
    let inner = records.iter().find(|r| r.name == "inner").expect("inner recorded");

    // Outer inclusive >= inner inclusive; outer self ≈ outer inclusive
    // minus inner inclusive. We use generous bounds because timing is
    // jittery in CI.
    assert!(outer.total_inclusive >= inner.total_inclusive);
    assert!(outer.total_self < outer.total_inclusive);
    assert_eq!(outer.hits, 1);
    assert_eq!(inner.hits, 1);

    disable();
}

#[test]
fn disabled_spans_are_noops() {
    let _guard = GATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    disable();
    reset_thread_spans();

    let guard = ScopeGuard::enter("ghost");
    assert!(!guard.is_active());
    drop(guard);

    assert!(take_thread_spans().is_empty());
}

#[test]
fn repeated_hits_accumulate() {
    let _guard = GATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    enable();
    reset_thread_spans();

    for _ in 0..3 {
        span("loop_body", || {
            std::thread::sleep(Duration::from_millis(1));
        });
    }

    let records = take_thread_spans();
    let loop_body = records.iter().find(|r| r.name == "loop_body").unwrap();
    assert_eq!(loop_body.hits, 3);

    disable();
}
