## 简单广播聊天服务

### 01 错误处理
需求：使用 thiserror 进行错误处理（为你定义的 error 实现 IntoResponse）

```rust
// 自定义错误
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ShortenerError {
    ... ...
}


// 实现 IntoResponse
impl IntoResponse for ShortenerError {
    fn into_response(self) -> Response {
        ... ...
    }
}


// 修改错误返回类型
async fn shorten(
    State((state, listen_addr)): State<(AppState, String)>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, ShortenerError>;


// 修改错误返回类型
async fn redirect(
    Path(id): Path<String>,
    State((state, _)): State<(AppState, String)>,
) -> Result<impl IntoResponse, ShortenerError> ;
```

### 02 url 验证

实现过程：
```rust
fn validate_url(url: &str) -> Result<(), ShortenerError> {
    let parsed_url =
        url::Url::parse(url).map_err(|_| ShortenerError::InvalidUrl(url.to_string()))?;
    if !["http", "https"].contains(&parsed_url.scheme()) {
        return Err(ShortenerError::InvalidUrl(url.to_string()));
    }
    Ok(())
}


async fn shorten(&self, url: &str) -> Result<ShortenRes, ShortenerError> {
    // 验证 url 是否合法
    validate_url(url)?;
    ... ...
}
```

测试用例：
```
POST http://localhost:9876/
Content-Type: application/json

{
    "url": "error_url://error_url"
}
```

### 03 数据库连接池配置
使用 PgPoolOptions 来配置数据库连接池
```rust
impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let db = PgPoolOptions::new().max_connections(5).connect(url).await?;
        ... ...
    }
}
```
### 04 命令行参数
使用 clap 接收命令行输入的 database_url 和 listen_addr
```rust
#[derive(Clone, Debug, Parser)]
#[command(name="url", version, author, about, long_about = None)]
struct Config {
    // 数据库连接
    #[arg(
        long,
        default_value = "postgres://postgres:postgres@192.168.1.9:5432/shortener",
        help = "database url"
    )]
    database_url: String,

    // 监听地址
    #[arg(long, default_value = "0.0.0.0:9876", help = "listen address")]
    listen_addr: String,
}

async fn main() -> Result<()> {
    ... ...
    let config = Config::parse();
    ... ...
}
```
