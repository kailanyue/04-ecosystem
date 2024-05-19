use anyhow::Context;
use ecosystem::MyError;
use std::{fs, mem::size_of, num::ParseIntError};

fn main() -> Result<(), anyhow::Error> {
    // 返回错误的大小
    println!("size of anyhow::Error is {}", size_of::<anyhow::Error>());
    println!("size of std::io::Error is {}", size_of::<std::io::Error>());
    println!(
        "size of std::num::ParseIntError is {}",
        size_of::<ParseIntError>()
    );
    println!(
        "size of serde_json::Error is {}",
        size_of::<serde_json::Error>()
    );
    println!("size of string is {}", size_of::<String>());
    println!("size of MyError is {}", size_of::<MyError>());

    let path = "non-existent-file.txt";
    // context 适合需要为错误提供固定的、静态的上下文描述时
    let _contents = fs::read_to_string(path).context("Failed to read")?;
    // with_context 是惰性执行，适合上下文信息可能计算成本较高，或者只在错误发生时才需要时
    let _contents = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path))?;

    fail_with_error()?;

    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
