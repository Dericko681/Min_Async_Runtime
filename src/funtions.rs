use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use crate::components::{MiniRuntime, Task, JoinHandle};

pub fn spawn<F>(future: F) -> JoinHandle<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let (sender, receiver) = async_channel::bounded(1);
    let runtime = CURRENT_RUNTIME.with(|rt| {
        rt.lock().unwrap().clone()
    });
    
    let task = Task {
        future: Box::pin(async move {
            future.await;
            let _ = sender.send(()).await;
        }),
        waker: None,
    };
    runtime.task_queue.lock().unwrap().push_back(task);
    JoinHandle { receiver }
}

pub async fn sleep(duration: Duration) {
    let runtime = CURRENT_RUNTIME.with(|rt| {
        rt.lock().unwrap().clone()
    });
    let timer = runtime.timer.clone();
    timer.sleep(duration).await;
}

pub async fn yield_now() {
    let runtime = CURRENT_RUNTIME.with(|rt| rt.clone());
    {
        if let Ok(runtime_guard) = runtime.lock() {
            if let Ok(mut queue) = runtime_guard.task_queue.lock() {
                queue.push_back(Task {
                    future: Box::pin(async {}),
                    waker: None,
                });
            }
        }
        
    }
}


// Thread-local storage for the runtime
thread_local! {
    static CURRENT_RUNTIME: Arc<Mutex<MiniRuntime>> = Arc::new(Mutex::new(MiniRuntime::new()));
}