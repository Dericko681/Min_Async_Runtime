use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use crate::components::{MiniRuntime, Task, JoinHandle, Timer, Sleep};
use std::sync::{Arc, Mutex};
use crate::runtime_storage::TASK_QUEUE;

thread_local! {
    pub static RUNTIME: Arc<Mutex<MiniRuntime>> = Arc::new(Mutex::new(MiniRuntime::new()));
}

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
    TASK_QUEUE.with(|queue| {
        queue.lock().unwrap().push_back(task);
    });

    JoinHandle {
        future: Box::pin(async move {
            receiver.recv().unwrap()
        }),
    }
}

pub fn sleep(duration: Duration) -> Sleep {
    Timer::new().sleep(duration)
}

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