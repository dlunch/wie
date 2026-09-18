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

        let task_id = {
            let mut inner = self.inner.lock();
            inner.last_task_id += 1;
            inner.last_task_id
        };

        self.inner.lock().tasks.insert(task_id, Box::pin(fut));
        self.wake.wake_by_ref();

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

    fn step(&mut self, now: Instant) -> Result<()> {
        self.inner.lock().last_now = now;

        let mut next_tasks = HashMap::new();
        let tasks = self.inner.lock().tasks.drain().collect::<HashMap<_, _>>();
        let mut sleeping_tasks = self.inner.lock().sleeping_tasks.drain().collect::<HashMap<_, _>>();

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
            self.inner.lock().current_task_id = Some(task_id);

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

        self.inner.lock().sleeping_tasks.extend(sleeping_tasks);
        self.inner.lock().tasks.extend(next_tasks);

        if let Some(err) = first_error { Err(err) } else { Ok(()) }
    }

    pub(crate) fn sleep(&self, timeout: u64) {
        let task_id = self.inner.lock().current_task_id.unwrap();

        let until = self.inner.lock().last_now + timeout;
        self.inner.lock().sleeping_tasks.insert(task_id, until);
        self.wake.wake_by_ref();
    }
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;
    use core::{
        cell::Cell,
        future::{Future, poll_fn},
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
