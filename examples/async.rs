use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

async fn expensive_blocking_task(s: &str) -> String {
    sleep(Duration::from_millis(1000)).await;
    blake3::hash(s.as_bytes()).to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let ret = expensive_blocking_task("Hello").await;
    println!("main ret: {}", ret);
    // time::sleep(Duration::from_millis(2000)).await;

    println!("Hello, world!");
    Ok(()) // do not forget
}
