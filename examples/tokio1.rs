use std::{thread, time::Duration};

use tokio::{fs, runtime::Builder, time};

fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        rt.spawn(async {
            println!("Further1");
            let content = fs::read_to_string("Cargo.toml").await.unwrap();
            println!("content length: {}", content.len());
        });

        rt.spawn(async {
            println!("Further2");
            let ret = expensive_block_task("Hello");
            println!("ret: {}", ret);
        });

        rt.block_on(async {
            { time::sleep(Duration::from_nanos(1)) }.await;
        })
    });

    handle.join().unwrap();
}

fn expensive_block_task(s: &str) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}
