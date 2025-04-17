mod components;
mod funtions;
mod macros;
mod runtime;
mod timer;

use crate::components::MiniRuntime;
use crate::funtions::{sleep, spawn};
use std::time::Duration;

async fn task_one() {
    println!("task one: start");
    sleep(Duration::from_secs(1)).await;
    println!("task one: done");
}

async fn task_two() {
    println!("task two: start");
    sleep(Duration::from_secs(2)).await;
    println!("task two: done");
}

// Using block_on
fn main() {
    println!("welcome");
    let mut rt = MiniRuntime::new();
    rt.block_on(async {
        println!("test1");
        let _ = spawn(async {
            println!("Runtime started...");
        })
        .await;
        task_one().await;
        task_two().await;
    });
}

