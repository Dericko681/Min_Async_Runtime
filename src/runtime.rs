use std::{
    collections::VecDeque,
    future::Future,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake},
};
use crate::components::{MiniRuntime, Task};
use crate::runtime_storage::TASK_QUEUE;

pub struct TaskWaker;

impl TaskWaker {
    pub fn new() -> Self {
        TaskWaker
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        // When woken, add a dummy task to the queue to ensure the runtime continues
        TASK_QUEUE.with(|queue| {
            queue.lock().unwrap().push_back(Task {
                future: Box::pin(async {}),
                waker: None,
            });
        });
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.clone().wake();
    }
}

impl MiniRuntime {
    pub fn new() -> Self {
        MiniRuntime {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        // Pin the future to the stack
        let mut future = Box::pin(future);
        
        // Create a waker that will wake the main task
        let waker = Arc::new(TaskWaker).into();
        let mut cx = Context::from_waker(&waker);

        // Poll the future until it completes
        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(output) => return output,
                Poll::Pending => {
                    // Process any tasks in the queue
                    self.process_tasks();
                }
            }
        }
    }

    fn process_tasks(&self) {
        // Get the thread-local queue
        let mut queue = VecDeque::new();
        TASK_QUEUE.with(|q| {
            std::mem::swap(&mut *q.lock().unwrap(), &mut queue);
        });
        
        // Process each task
        while let Some(mut task) = queue.pop_front() {
            let waker = Arc::new(TaskWaker).into();
            let mut cx = Context::from_waker(&waker);

            match task.future.as_mut().poll(&mut cx) {
                Poll::Ready(()) => {
                    // Task completed
                }
                Poll::Pending => {
                    // Task not ready, put it back in the queue
                    queue.push_back(task);
                }
            }
        }

        // Put any remaining tasks back in the queue
        TASK_QUEUE.with(|q| {
            std::mem::swap(&mut *q.lock().unwrap(), &mut queue);
        });
    }
} 