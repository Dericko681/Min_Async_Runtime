use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
    sync::{Arc, Mutex},
    collections::VecDeque,
};

#[derive(Clone)]
pub struct Timer {
    pub wakeups: Arc<Mutex<VecDeque<(Instant, std::task::Waker)>>>,
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