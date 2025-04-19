mod components;
mod runtime;
mod runtime_storage;
mod funtions;

use std::time::Duration;
use crate::components::MiniRuntime;
use crate::funtions::{spawn, sleep};

async fn example_task(id: u32) {
    println!("Task {} started", id);
    spawn(async move {
        println!("Nested task {} started", id);
        sleep(Duration::from_millis(100)).await;
        println!("Nested task {} completed", id);
    });
    println!("Task {} completed", id);
}

fn main() {
    let mut runtime = MiniRuntime::new();
    
    // Spawn some example tasks
    spawn(example_task(1));
    spawn(example_task(2));
    spawn(example_task(3));
    spawn(example_task(4));
    
    // Run the runtime
    runtime.block_on(async {
        println!("Main task started");
        sleep(Duration::from_millis(200)).await;
        println!("Main task completed");
    });
}