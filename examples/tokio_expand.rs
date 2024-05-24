use std::{thread, time::Duration};

use anyhow::Result;
use tokio::{
    fs,
    runtime::{Builder, Runtime},
    time::sleep,
};

// [tokio::main] 的展开

// #[tokio::main]
fn main() -> Result<()> {
    let handle = thread::spawn(|| {
        // 使用 multi thread runtime
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();
        rt.block_on(run(&rt));
    });

    handle.join().unwrap();

    Ok(())
}

fn expensive_blocking_task(s: &str) -> String {
    thread::sleep(Duration::from_millis(1000));
    blake3::hash(s.as_bytes()).to_string()
}

async fn run(rt: &Runtime) {
    rt.spawn(async {
        println!("future 2");
        let result = expensive_blocking_task("hello");
        println!("result: {:?}", result);
    });

    rt.spawn(async {
        println!("future 1");
        let content = fs::read("Cargo.toml").await.unwrap();
        let content = String::from_utf8(content).unwrap();
        let tomt_value: toml::Value = toml::from_str(&content).unwrap();
        let json_content = serde_json::to_string_pretty(&tomt_value).unwrap();
        println!("content: {:}", json_content);
    });

    sleep(Duration::from_secs(1)).await;
}
