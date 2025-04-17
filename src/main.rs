mod macros; mod components; mod runtime; mod timer;
mod funtions;

use std::time::Duration;
use crate::components::MiniRuntime;
use crate::funtions::{sleep, spawn};

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
        let _ = spawn(async {
            println!("Runtime started...");
        }).await;
        task_one().await;
        task_two().await;
    });
}

// Remove the macro version of main to avoid duplicate main functions
// mini_rt! {
//     async fn main() {
//         let _ = spawn(async {
//             println!("Runtime started...");
//         }).await;
//         task_one().await;
//         task_two().await;
//     }
// }