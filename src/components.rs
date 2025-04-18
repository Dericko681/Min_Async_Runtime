use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct Timer {
    pub wakeups: Arc<Mutex<VecDeque<(Instant, Waker)>>>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            wakeups: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn sleep(&self, duration: Duration) -> Sleep {
        Sleep {
            timer: self.clone(),
            duration,
            registered: false,
        }
    }
    
    pub fn check_wakeups(&self) {
        let mut wakeups = self.wakeups.lock().unwrap();
        let now = Instant::now();
        while let Some((time, _waker)) = wakeups.front() {
            if *time <= now {
                let (_, waker) = wakeups.pop_front().unwrap();
                waker.wake();
            } else {
                break;
            }
        }
    }
}

pub struct Sleep {
    timer: Timer,
    duration: Duration,
    registered: bool,
}

impl Future for Sleep {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.registered {
            let wake_at = Instant::now() + self.duration;
            self.timer.wakeups.lock().unwrap().push_back((wake_at, cx.waker().clone()));
            self.registered = true;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

// The main runtime struct
#[derive(Clone)]
pub struct MiniRuntime {
    pub task_queue: Arc<Mutex<VecDeque<Task>>>,
}

// A task that can be scheduled on the executor
pub struct Task {
    pub future: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub waker: Option<Waker>,
}

// JoinHandle for spawned tasks
pub struct JoinHandle<T> {
    pub future: Pin<Box<dyn Future<Output = T> + Send>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.future.as_mut().poll(cx)
    }
}

// Helper function to spawn a new task
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (sender, receiver) = std::sync::mpsc::channel();
    let future = async move {
        let output = future.await;
        let _ = sender.send(output);
    };

    let task = Task {
        future: Box::pin(future),
        waker: None,
    };

    // Add the task to the runtime's queue
    crate::runtime_storage::TASK_QUEUE.with(|queue| {
        queue.lock().unwrap().push_back(task);
    });

    JoinHandle {
        future: Box::pin(async move {
            receiver.recv().unwrap()
        }),
    }
}

// Helper function to yield the current task
pub async fn yield_now() {
    struct YieldNow {
        yielded: bool,
    }

    impl Future for YieldNow {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    YieldNow { yielded: false }.await
}