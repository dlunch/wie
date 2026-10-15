use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use test_utils::{TestClock, TestPlatform};
use wie_backend::{DefaultTaskRunner, System};
use wie_util::Result;

#[test]
fn sleep_waits_from_the_call_time_after_task_work() -> Result<()> {
    let clock = TestClock::new();
    clock.set(1000);
    let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "test", "test", DefaultTaskRunner);
    let completed = Arc::new(AtomicBool::new(false));
    let task_system = system.clone();
    let task_clock = clock.clone();
    let task_completed = completed.clone();
    system.spawn(async move || {
        // Simulate work between the executor's poll and the sleep request.
        task_clock.advance(35);
        task_system.sleep(115).await;
        task_completed.store(true, Ordering::SeqCst);
        Ok(())
    });

    system.tick()?;
    for now in [1115, 1149] {
        clock.set(now);
        system.tick()?;
        assert!(!completed.load(Ordering::SeqCst), "sleep resumed early at {now}");
    }
    clock.set(1150);
    system.tick()?;
    assert!(completed.load(Ordering::SeqCst));
    Ok(())
}
