use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::components::Task;

thread_local! {
    pub static TASK_QUEUE: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));
} 