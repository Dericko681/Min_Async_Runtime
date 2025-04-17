use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake},
    thread,
};
use crate::components::{MiniRuntime, Task, Timer};

impl MiniRuntime {
    pub fn new() -> Self {
        MiniRuntime {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            timer: Timer {
                wakeups: Arc::new(Mutex::new(VecDeque::new())),
            },
        }
    }

    pub fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        let waker = Arc::new(TaskWaker {
            task_queue: self.task_queue.clone(),
        }).into();
        let mut cx = Context::from_waker(&waker);
        
        let mut future = Box::pin(future);
        loop {
            // Process the main future
            if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
                return output;
            }
            
            // Process all ready tasks
            self.process_tasks();
            
            // Check timer
            self.timer.check_wakeups();
            
            // Yield if no progress
            if self.task_queue.lock().unwrap().is_empty() {
                thread::yield_now();
            }
        }
    }

    fn process_tasks(&self) {
        let mut queue = self.task_queue.lock().unwrap();
        let mut i = 0;
        while i < queue.len() {
            let mut task = queue.pop_front().unwrap();
            let waker = task.waker.take().unwrap_or_else(|| {
                Arc::new(TaskWaker {
                    task_queue: self.task_queue.clone(),
                }).into()
            });
            
            let mut cx = Context::from_waker(&waker);
            match task.future.as_mut().poll(&mut cx) {
                Poll::Ready(()) => continue,
                Poll::Pending => {
                    task.waker = Some(waker);
                    queue.push_back(task);
                    i += 1;
                }
            }
        }
    }
}

// Waker implementation
struct TaskWaker {
    task_queue: Arc<Mutex<VecDeque<Task>>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    
    fn wake_by_ref(self: &Arc<Self>) {
        // In a real implementation, we'd wake the executor
    }
}