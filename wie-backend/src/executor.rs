use alloc::{boxed::Box, sync::Arc, task::Wake};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, Waker},
};

use hashbrown::HashMap;
use spin::Mutex;

use wie_util::{Result, WieError};

use crate::time::Instant;

type Task = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

struct TaskWake(AtomicBool);

impl Wake for TaskWake {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.store(true, Ordering::Release);
    }
}

pub struct ExecutorInner {
    closed: bool,
    current_task_id: Option<usize>,
    tasks: HashMap<usize, Task>,
    sleeping_tasks: HashMap<usize, Instant>,
    last_task_id: usize,
    last_now: Instant,
}

pub trait AsyncCallable<R>: Send
where
    R: Send,
{
    fn call(self) -> impl Future<Output = R> + Send;
}

impl<F, R, Fut> AsyncCallable<R> for F
where
    F: FnOnce() -> Fut + 'static + Send,
    R: AsyncCallableResult,
    Fut: Future<Output = R> + 'static + Send,
{
    async fn call(self) -> R {
        self().await
    }
}

pub trait AsyncCallableResult: Send {
    fn err(self) -> Option<WieError>;
}

impl<R> AsyncCallableResult for core::result::Result<R, WieError>
where
    R: Send,
{
    fn err(self) -> Option<WieError> {
        self.err()
    }
}

impl AsyncCallableResult for () {
    fn err(self) -> Option<WieError> {
        None
    }
}

#[derive(Clone)]
pub struct Executor {
    inner: Arc<Mutex<ExecutorInner>>,
    wake: Arc<TaskWake>,
}

impl Executor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(ExecutorInner {
            closed: false,
            current_task_id: None,
            tasks: HashMap::new(),
            sleeping_tasks: HashMap::new(),
            last_task_id: 0,
            last_now: Instant::from_epoch_millis(0),
        }));

        Self {
            inner,
            wake: Arc::new(TaskWake(AtomicBool::new(false))),
        }
    }

    pub fn spawn<C, R>(&self, callable: C) -> usize
    where
        C: AsyncCallable<R> + 'static,
        R: AsyncCallableResult,
    {
        let fut = async move {
            let result = callable.call().await;
            if let Some(err) = result.err() {
                return Err(err);
            }

            Ok(())
        };

        let mut inner = self.inner.lock();
        inner.last_task_id += 1;
        let task_id = inner.last_task_id;
        if !inner.closed {
            inner.tasks.insert(task_id, Box::pin(fut));
            self.wake.wake_by_ref();
        }
        drop(inner);

        task_id
    }

    // TODO we need to remove error handling from here. we need to JoinHandle like on spawn..
    pub fn tick<T>(&mut self, now: T) -> Result<()>
    where
        T: Fn() -> Instant,
    {
        let end = now() + 8; // TODO hardcoded
        loop {
            let now = now();

            if now > end {
                break;
            }

            {
                let inner = self.inner.lock();
                if inner.closed {
                    break;
                }
                let running_task_count = inner.tasks.len() - inner.sleeping_tasks.len();
                if running_task_count == 0 && !inner.sleeping_tasks.is_empty() {
                    let next_wakeup = *inner.sleeping_tasks.values().min().unwrap();
                    if now < next_wakeup {
                        break;
                    }
                }
            }

            self.wake.0.store(false, Ordering::Release);
            self.step(now)?;
            if !self.wake.0.load(Ordering::Acquire) {
                break;
            }
        }

        Ok(())
    }

    pub fn current_task_id(&self) -> u64 {
        self.inner.lock().current_task_id.unwrap() as _
    }

    pub fn shutdown(&self) {
        let tasks = {
            let mut inner = self.inner.lock();
            inner.closed = true;
            inner.sleeping_tasks.clear();
            core::mem::take(&mut inner.tasks)
        };
        // Task destructors may reenter the executor.
        drop(tasks);
    }

    fn step(&mut self, now: Instant) -> Result<()> {
        let (tasks, mut sleeping_tasks) = {
            let mut inner = self.inner.lock();
            if inner.closed {
                return Ok(());
            }
            inner.last_now = now;
            (core::mem::take(&mut inner.tasks), core::mem::take(&mut inner.sleeping_tasks))
        };

        let mut next_tasks = HashMap::new();

        let mut first_error = None;
        let waker = Waker::from(self.wake.clone());

        for (task_id, mut task) in tasks.into_iter() {
            let item = sleeping_tasks.get(&task_id);
            if let Some(item) = item {
                if *item <= now {
                    sleeping_tasks.remove(&task_id);
                } else {
                    next_tasks.insert(task_id, task);
                    continue;
                }
            }

            let mut context = Context::from_waker(&waker);
            {
                let mut inner = self.inner.lock();
                if inner.closed {
                    break;
                }
                inner.current_task_id = Some(task_id);
            }

            match task.as_mut().poll(&mut context) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(err)) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
                Poll::Pending => {
                    next_tasks.insert(task_id, task);
                }
            }

            self.inner.lock().current_task_id = None;
        }

        {
            let mut inner = self.inner.lock();
            if !inner.closed {
                inner.sleeping_tasks.extend(sleeping_tasks);
                inner.tasks.extend(next_tasks);
            }
        }

        if let Some(err) = first_error { Err(err) } else { Ok(()) }
    }

    pub(crate) fn sleep(&self, timeout: u64) {
        let mut inner = self.inner.lock();
        if inner.closed {
            return;
        }
        let task_id = inner.current_task_id.unwrap();
        let until = inner.last_now + timeout;
        inner.sleeping_tasks.insert(task_id, until);
        self.wake.wake_by_ref();
    }
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;
    use core::{
        cell::Cell,
        future::{Future, pending, poll_fn},
        pin::Pin,
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
        task::{Context, Poll},
    };

    use wie_util::WieError;

    use super::Executor;
    use crate::time::Instant;

    struct YieldOnce(bool);

    impl Future for YieldOnce {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.0 {
                Poll::Ready(())
            } else {
                self.0 = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    fn advancing_clock(start: u64) -> impl Fn() -> Instant {
        let time = Cell::new(start);
        move || {
            let now = time.get();
            time.set(now + 1);
            Instant::from_epoch_millis(now)
        }
    }

    struct SpawnOnDrop {
        executor: Executor,
        resource: Arc<()>,
        dropped: Arc<AtomicUsize>,
    }

    impl Drop for SpawnOnDrop {
        fn drop(&mut self) {
            assert!(self.executor.inner.try_lock().is_some(), "task dropped under executor lock");
            self.executor.shutdown();
            let executor = self.executor.clone();
            let resource = self.resource.clone();
            self.executor.spawn(move || async move {
                pending::<()>().await;
                drop((executor, resource));
            });
            self.executor.clone().tick(advancing_clock(0)).unwrap();
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn pending_tasks_only_repeat_within_a_tick_when_woken() {
        for wakes in [0, 2] {
            let mut executor = Executor::new();
            let polls = Arc::new(AtomicUsize::new(0));
            let observed = polls.clone();
            executor.spawn(move || async move {
                poll_fn(|cx| {
                    if observed.fetch_add(1, Ordering::Relaxed) < wakes {
                        cx.waker().wake_by_ref();
                    }
                    Poll::<()>::Pending
                })
                .await;
            });
            executor.tick(advancing_clock(0)).unwrap();
            assert_eq!(polls.load(Ordering::Relaxed), wakes + 1);
            executor.tick(advancing_clock(100)).unwrap();
            assert_eq!(polls.load(Ordering::Relaxed), wakes + 2);
        }
    }

    #[test]
    fn zero_duration_sleeps_and_newly_spawned_tasks_resume_in_the_same_tick() {
        let mut executor = Executor::new();
        let completed = Arc::new(AtomicBool::new(false));
        let observed = completed.clone();
        let task_executor = executor.clone();
        executor.spawn(move || async move {
            crate::task::SleepFuture::new(0, &task_executor).await;
            task_executor.spawn(move || async move {
                observed.store(true, Ordering::Relaxed);
            });
        });
        executor.tick(advancing_clock(0)).unwrap();
        assert!(completed.load(Ordering::Relaxed));
    }

    #[test]
    fn test_shutdown_releases_queued_task_cycle() {
        let executor = Executor::new();
        let weak_executor = Arc::downgrade(&executor.inner);
        let resource = Arc::new(());
        let weak_resource = Arc::downgrade(&resource);
        let task_executor = executor.clone();
        executor.spawn(move || async move {
            pending::<()>().await;
            drop((task_executor, resource));
        });

        assert!(weak_resource.upgrade().is_some());
        executor.shutdown();
        assert!(weak_resource.upgrade().is_none());
        drop(executor);
        assert!(weak_executor.upgrade().is_none());
    }

    #[test]
    fn test_shutdown_cancels_pending_and_sleeping_tasks() {
        let mut executor = Executor::new();
        let resources = [Arc::new(()), Arc::new(())];
        let weak_resources = resources.each_ref().map(Arc::downgrade);
        let completed = Arc::new(AtomicBool::new(false));
        for (sleeping, resource) in [false, true].into_iter().zip(resources) {
            let task_executor = executor.clone();
            let completed = completed.clone();
            executor.spawn(move || async move {
                if sleeping {
                    task_executor.sleep(100);
                }
                YieldOnce(false).await;
                completed.store(true, Ordering::Relaxed);
                drop(resource);
            });
        }

        executor.step(Instant::from_epoch_millis(0)).unwrap();
        assert!(weak_resources.iter().all(|resource| resource.upgrade().is_some()));
        assert_eq!(executor.inner.lock().sleeping_tasks.len(), 1);
        executor.shutdown();
        executor.shutdown();
        assert!(weak_resources.iter().all(|resource| resource.upgrade().is_none()));
        assert!(executor.inner.lock().sleeping_tasks.is_empty());
        executor.tick(advancing_clock(200)).unwrap();
        assert!(!completed.load(Ordering::Relaxed));
    }

    #[test]
    fn test_shutdown_during_poll_does_not_resurrect_drained_tasks() {
        for fail in [false, true] {
            let mut executor = Executor::new();
            let resource = Arc::new(());
            let weak_resource = Arc::downgrade(&resource);
            let sleeper_resource = resource.clone();
            let sleeper_executor = executor.clone();
            executor.spawn::<_, ()>(move || async move {
                sleeper_executor.sleep(100);
                YieldOnce(false).await;
                drop(sleeper_resource);
                panic!("sleeping task resumed after shutdown");
            });
            executor.step(Instant::from_epoch_millis(0)).unwrap();

            let polls = Arc::new(AtomicUsize::new(0));
            let dropped = Arc::new(AtomicUsize::new(0));
            for _ in 0..3 {
                let task_executor = executor.clone();
                let polls = polls.clone();
                let on_drop = SpawnOnDrop {
                    executor: executor.clone(),
                    resource: resource.clone(),
                    dropped: dropped.clone(),
                };
                executor.spawn(move || async move {
                    let _on_drop = on_drop;
                    poll_fn(move |_| {
                        polls.fetch_add(1, Ordering::Relaxed);
                        let task_id = task_executor.current_task_id();
                        task_executor.shutdown();
                        assert_eq!(task_executor.current_task_id(), task_id);
                        task_executor.sleep(100);
                        task_executor.clone().tick(advancing_clock(0)).unwrap();
                        assert_eq!(task_executor.current_task_id(), task_id);
                        if fail {
                            Poll::Ready(Err::<(), _>(WieError::FatalError("shutdown error".into())))
                        } else {
                            Poll::Pending
                        }
                    })
                    .await
                });
            }
            drop(resource);

            let result = executor.step(Instant::from_epoch_millis(1));
            if fail {
                assert!(matches!(result, Err(WieError::FatalError(message)) if message == "shutdown error"));
            } else {
                result.unwrap();
            }
            assert_eq!(polls.load(Ordering::Relaxed), 1);
            assert_eq!(dropped.load(Ordering::Relaxed), 3);
            assert!(weak_resource.upgrade().is_none());
            assert!(executor.inner.lock().tasks.is_empty());
            assert!(executor.inner.lock().sleeping_tasks.is_empty());
            assert!(executor.inner.lock().current_task_id.is_none());
            executor.tick(advancing_clock(200)).unwrap();
            assert_eq!(polls.load(Ordering::Relaxed), 1);
        }
    }

    #[test]
    fn test_shutdown_and_rejected_spawn_drop_outside_locks() {
        for closed in [false, true] {
            let executor = Executor::new();
            let weak_executor = Arc::downgrade(&executor.inner);
            let resource = Arc::new(());
            let weak_resource = Arc::downgrade(&resource);
            let dropped = Arc::new(AtomicUsize::new(0));
            let on_drop = SpawnOnDrop {
                executor: executor.clone(),
                resource,
                dropped: dropped.clone(),
            };
            if closed {
                executor.shutdown();
            }
            let task_id = executor.spawn(move || async move {
                pending::<()>().await;
                drop(on_drop);
            });
            if !closed {
                executor.shutdown();
            }
            assert_eq!(dropped.load(Ordering::Relaxed), 1);
            assert!(weak_resource.upgrade().is_none());
            assert!(executor.spawn(|| async {}) > task_id);
            drop(executor);
            assert!(weak_executor.upgrade().is_none());
        }
    }

    #[test]
    fn test_failed_task_preserves_others() {
        let mut executor = Executor::new();

        executor.spawn(|| async { Err::<(), _>(WieError::FatalError("test error".into())) });

        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();
        executor.spawn(move || async move {
            YieldOnce(false).await;
            completed_clone.store(true, Ordering::Relaxed);
        });

        assert!(executor.tick(advancing_clock(0)).is_err());
        assert!(!completed.load(Ordering::Relaxed));

        executor.tick(advancing_clock(100)).unwrap();
        assert!(completed.load(Ordering::Relaxed));
    }

    #[test]
    fn test_failed_task_preserves_sleeping_tasks() {
        let mut executor = Executor::new();

        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();
        let executor_clone = executor.clone();
        executor.spawn(move || async move {
            executor_clone.sleep(100);
            YieldOnce(false).await;
            completed_clone.store(true, Ordering::Relaxed);
        });

        executor.spawn(|| async { Err::<(), _>(WieError::FatalError("test error".into())) });

        assert!(executor.tick(advancing_clock(0)).is_err());
        assert!(!completed.load(Ordering::Relaxed));

        executor.tick(advancing_clock(50)).unwrap();
        assert!(!completed.load(Ordering::Relaxed));

        executor.tick(advancing_clock(200)).unwrap();
        assert!(completed.load(Ordering::Relaxed));
    }

    #[test]
    fn test_all_ok_tasks_complete() {
        let mut executor = Executor::new();

        let completed_a = Arc::new(AtomicBool::new(false));
        let completed_a_clone = completed_a.clone();
        executor.spawn(move || async move {
            completed_a_clone.store(true, Ordering::Relaxed);
        });

        let completed_b = Arc::new(AtomicBool::new(false));
        let completed_b_clone = completed_b.clone();
        executor.spawn(move || async move {
            YieldOnce(false).await;
            completed_b_clone.store(true, Ordering::Relaxed);
        });

        executor.tick(advancing_clock(0)).unwrap();

        assert!(completed_a.load(Ordering::Relaxed));
        assert!(completed_b.load(Ordering::Relaxed));
    }
}
