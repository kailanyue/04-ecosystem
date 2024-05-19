## 召唤元素：Rust 生态系统概览
### 4.1 错误处理
错误处理：anyhow、thiserror, snafu
- anyhow：统一，简单的错误处理，适用于应用程序级别
- thiserror：自定义，丰富的错误处理，适用于库级别
- snafu：更细力度地管理错误
- https://github.com/kube-rs/kube/discussions/453
- 需要注意 Result<T, E> 的大小

#### 4.1.1 anyhow
`anyhow` 库提供了一个不透明的错误类型 `anyhow::Error`，它实现了 `Error` trait。这个库主要用于应用程序（applications），当调用者不需要关心错误的具体细节，只是需要将错误传递到日志记录系统或者上层代码。`anyhow` 使得错误处理变得更加灵活，可以轻松地添加上下文信息到错误中。

##### **作用**：Use `Result<T, anyhow::Error>`, or equivalently `anyhow::Result<T>`, as the return type of any fallible function.

##### **附加上下文**：use `context()` or `with_context(||format!(""))` ,attach context to help the person troubleshooting the error understand where things went wrong.

`context` 和 `with_context` 是 `anyhow` 提供的两个方法，用于在处理错误时添加上下文信息。让我们详细了解它们的作用、区别和应用场景：

1. **`context` 方法**：
    - **作用**：`context` 方法用于将错误值包装在附加的上下文中。这意味着你可以为错误提供更详细的描述，以便更好地理解错误的来源。
    - **用法**：你可以使用 `context` 方法来添加静态的上下文信息。例如：

        ```rust
        use anyhow::{Context, Result};

        fn read_config_file(path: &str) -> Result<String> {
            std::fs::read_to_string(path)
                .context(format!("Failed to read file from {}", path))
        }
        ```

    - **应用场景**：
        - 当你需要为错误提供固定的、静态的上下文描述时，可以使用 `context` 方法。

2. **`with_context` 方法**：
    - **作用**：`with_context` 方法也用于将错误值包装在附加的上下文中，但它的特点是**惰性求值**。这意味着只有在错误发生时，才会计算上下文信息。
    - **用法**：你需要传递一个闭包，该闭包在错误发生时计算上下文信息。例如：

        ```rust
        use anyhow::{Context, Result};

        fn read_config_file_lazy(path: &str) -> Result<String> {
            std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read file from {}", path))
        }
        ```

    - **应用场景**：
        - 当你的上下文信息可能计算成本较高，或者只在错误发生时才需要时，可以使用 `with_context` 方法。
        - 例如，如果计算上下文需要耗费时间，你可以使用 `with_context` 来避免不必要的计算，从而提高性能。

总结一下：
- `context` 方法适用于静态的上下文信息，而 `with_context` 方法适用于惰性求值的上下文信息。
- 根据具体需求，你可以选择使用其中之一来增强错误处理并提供更有用的错误信息。

##### **Downcasting**: Downcasting is supported and can be by value, by shared reference, or by mutable reference as needed.

##### **One-off error messages**：One-off error messages can be constructed using the anyhow! macro
#### 4.1.2 thiserror
`thiserror` 库提供了一个派生宏（derive macro），用于简化用户定义错误类型上的 `Error` trait 的实现。它主要用于库（libraries）中，当调用者需要关心错误的具体细节时，比如错误是一个枚举（enum），每个变种都需要不同的处理方式。`thiserror` 允许创建自定义的错误类型，这些类型可以很容易地转换成其他错误类型或者被其他错误类型所包含。
##### **支持的类型**：Errors may be enums, structs with named fields, tuple structs, or unit structs.

##### **Display impl**：A `Display` impl is generated for your error if you provide `#[error("...")]` messages on the struct or each variant of your enum

  The messages support a shorthand for interpolating fields from the error.
    - `#[error("{var}")]`&ensp;⟶&ensp;`write!("{}", self.var)`
    - `#[error("{0}")]`&ensp;⟶&ensp;`write!("{}", self.0)`
    - `#[error("{var:?}")]`&ensp;⟶&ensp;`write!("{:?}", self.var)`
    - `#[error("{0:?}")]`&ensp;⟶&ensp;`write!("{:?}", self.0)`

##### **From impl**：A `From` impl is generated for each variant containing a `#[from]` attribute.

#### 4.1.3 thiserror 和 anyhow
总结一下它们的异同和联系：
- **相同点**：两者都是用于处理 Rust 中的可恢复错误（recoverable errors）。
- **不同点**：
  - `thiserror` 适用于需要详细错误信息的场景，通常用于库开发。
  - `anyhow` 适用于错误信息不需要详细区分的场景，通常用于应用程序开发。
- **联系**：它们都与 Rust 的错误处理机制紧密相关，`thiserror` 用于定义错误，`anyhow` 用于处理错误。


#### 4.1.4 snafu
SNAFU is a library to easily assign underlying errors into domain-specific errors while adding context.

**Snafu** 是一个 Rust 的库，用于处理错误。它的目标是简化错误类型的创建和管理，特别是在定义自定义错误类型时。让我们来看看它的一些特点和应用场景：

1. **Turnkey 错误**：`snafu` 提供了一种轻松生成基于字符串的错误的方式。如果你只需要报告简单的错误消息，可以使用 `Whatever` 类型和 `whatever!` 宏。例如：

    ```rust
    use snafu::{prelude::*, Whatever};

    fn is_valid_id(id: u16) -> Result<(), Whatever> {
        if id < 10 {
            whatever!("ID may not be less than 10, but it was {id}");
        }
        Ok(())
    }
    ```

2. **自定义错误类型**：
    - **结构体风格**：如果你需要更复杂的错误类型，可以使用结构体来定义自己的错误。`snafu` 会根据结构体定义自动生成上下文选择器类型，用于提供更人性化的错误创建。例如：

        ```rust
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        #[snafu(display("ID may not be less than 10, but it was {id}"))]
        struct InvalidIdError {
            id: u16,
        }

        fn is_valid_id(id: u16) -> Result<(), InvalidIdError> {
            ensure!(id >= 10, InvalidIdSnafu { id });
            Ok(())
        }
        ```

    - **枚举风格**：如果你需要报告多种可能的错误，可以使用枚举。`snafu` 会根据枚举定义自动生成上下文选择器类型，用于提供更人性化的错误创建。例如：

        ```rust
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        enum Error {
            #[snafu(display("ID may not be less than 10, but it was {id}"))]
            InvalidId { id: u16 },
        }

        fn is_valid_id(id: u16) -> Result<(), Error> {
            ensure!(id >= 10, InvalidIdSnafu { id });
            Ok(())
        }
        ```

3. **添加上下文信息**：你可以为错误类型添加上下文信息，以便更好地理解错误的来源。例如，你可以包装底层的 I/O 错误：

    ```rust
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(display("Could not read file {path}"))]
    struct ConfigFileError {
        source: std::io::Error,
        path: String,
    }

    fn read_config_file(path: &str) -> Result<String, ConfigFileError> {
        std::fs::read_to_string(path).context(ConfigFileSnafu { path })
    }
    ```

总之，`snafu` 是一个强大且灵活的错误处理库，适用于库开发和应用程序开发，具体取决于你的需求。

#### 4.1.5 Result<T, E> 的大小
总结一下它们的异同和联系：
- **相同点**：两者都是用于处理 Rust 中的可恢复错误（recoverable errors）。
- **不同点**：
  - `thiserror` 适用于需要详细错误信息的场景，通常用于**库开发**。
  - `anyhow` 适用于错误信息不需要详细区分的场景，通常用于**应用程序开发**。
- **联系**：它们都与 Rust 的错误处理机制紧密相关，`thiserror` 用于定义错误，`anyhow` 用于处理错误。

应用场景示例：
```rust
// 使用 thiserror 创建自定义错误
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("网络错误")]
    Network(#[from] std::io::Error),
    #[error("数据解析错误")]
    Parse(#[from] serde_json::Error),
    // 其他错误变种
}

// 使用 anyhow 处理错误
use anyhow::{Result, Context};

fn do_something() -> Result<()> {
    let data = std::fs::read_to_string("file.txt")
        .context("读取文件失败")?;
    // ... 其他操作
    Ok(())
}
```
在这个例子中，`thiserror` 用于定义可能发生的不同错误类型，而 `anyhow` 用于在函数中处理这些错误，并提供额外的上下文信息。¹²

Source: Conversation with Bing, 2024/5/19
(1) thiserror, anyhow, or How I Handle Errors in Rust Apps. https://www.shakacode.com/blog/thiserror-anyhow-or-how-i-handle-errors-in-rust-apps/.
(2) thiserror and anyhow - Comprehensive Rust - google.github.io. https://google.github.io/comprehensive-rust/error-handling/thiserror-and-anyhow.html.
(3) thiserror, anyhow, or How I Handle Errors in Rust Apps. https://alexfedoseev.com/blog/post/thiserror-anyhow-or-how-i-handle-errors-in-rust-apps.
