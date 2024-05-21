use anyhow::Result;
use derive_more::{Add, Display, From, Into};

#[derive(Debug, PartialEq, Clone, Copy, From, Add, Into, Display)]
struct MyInt(i32);

#[derive(Debug, PartialEq, From, Into, Add)]
struct Point2D {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, From, Add, Display)]
enum MyEnum {
    #[display(fmt = "int: {}", _0)]
    Int(i32),
    Uint(u32),
    #[display(fmt = "nothing")]
    Nothing,
}

fn main() -> Result<()> {
    let my_int: MyInt = 10.into();
    let v = my_int + 20.into();
    let v1: i32 = v.into();

    println!("my_int: {}, v: {}, v1: {}", my_int, v, v1);

    let e: MyEnum = 10i32.into();
    let e1: MyEnum = 20u32.into();
    let e2 = MyEnum::Nothing;
    println!("e: {:?}, e1: {:?}, e2: {:?}", e, e1, e2);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_myint() {
        assert_eq!(MyInt(1) + MyInt(2), MyInt(3));
        assert_eq!(MyInt(11), MyInt(5) + 6.into());
    }

    #[test]
    fn test_point2d() {
        assert_eq!((5, 6), Point2D { x: 5, y: 6 }.into());
        assert_eq!(
            Point2D { x: 10, y: 12 },
            Point2D { x: 5, y: 6 } + Point2D { x: 5, y: 6 }
        );
    }

    #[test]
    fn test_myenum() {
        assert!(MyEnum::Int(15) == (MyEnum::Int(8) + 7.into()).unwrap());
        assert!(MyEnum::Int(15).to_string() == "int: 15");
        assert!(MyEnum::Uint(42).to_string() == "42");
        assert!(MyEnum::Nothing.to_string() == "nothing");
    }
}
