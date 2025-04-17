use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::Instant,
   
};
use futures_core::stream::Stream;

// The main runtime struct
#[derive(Clone)]
pub struct MiniRuntime {
    pub task_queue: Arc<Mutex<VecDeque<Task>>>,
    pub timer: Timer,
}

// A task that can be scheduled on the executor
pub struct Task {
    pub future: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub waker: Option<Waker>,
}

// Timer for sleep functionality
#[derive(Clone)]
pub struct Timer {
    pub wakeups: Arc<Mutex<VecDeque<(Instant, Waker)>>>,
}

// JoinHandle for spawned tasks
pub struct JoinHandle<T> {
    pub(crate) receiver: async_channel::Receiver<T>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let receiver = &mut self.get_unchecked_mut().receiver;
            match Stream::poll_next(Pin::new_unchecked(receiver), cx) {
                Poll::Ready(Some(val)) => Poll::Ready(val),
                Poll::Ready(None) => panic!("channel closed"),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}