use anyhow::Result;
use std::{thread, time::Duration};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(32);

    let handler = worker(rx);

    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            println!("Send task {i}");
            tx.send(format!("Hello, World {i}")).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    handler.join().unwrap();

    Ok(())
}

fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(s) = rx.blocking_recv() {
            let ret = expensive_block_task(&s);
            println!("result: {}", ret);
        }
    })
}

fn expensive_block_task(s: &str) -> String {
    thread::sleep(Duration::from_millis(900));
    blake3::hash(s.as_bytes()).to_string()
}
