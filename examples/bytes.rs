use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};

fn main() -> Result<()> {
    // 创建一个新的可变字节缓冲区
    let mut buf = BytesMut::with_capacity(1024);

    // 写入数据
    buf.extend_from_slice(b"hello world\n");
    buf.put(&b"hello rust\n"[..]);
    buf.put_i64(0xdeadbeef);

    // 读取数据
    println!("{:?}", buf);
    let a = buf.split();
    let mut b = a.freeze();

    let c = b.split_to(12);
    println!("{:?}", c);

    println!("{:?}", b);
    println!("{:?}", buf);
    println!("{:?}", c);

    let s = "Rust is a programming language";
    let b = Bytes::from(s);

    if let Some(index) = b.windows("is".len()).position(|window| window == b"is") {
        // 找到 "is" 的起始索引
        let split_index = index + "is".len();
        let (first_part, second_part) = b.split_at(split_index);

        // 打印结果
        println!("First part: {:?}", std::str::from_utf8(first_part)?);
        println!("Second part: {:?}", std::str::from_utf8(second_part)?);
    } else {
        println!("The substring 'is' was not found!");
    }

    Ok(())
}
