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
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, ShortenerError>;


// 修改错误返回类型
async fn redirect(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ShortenerError>;
```

### 02 url 验证

实现过程：
```rust
fn validate_url(url: &str) -> Result<(), ShortenerError> {
    url::Url::parse(url).map_err(|_| {
        warn!("Failed to parse URL: {url}");
        ShortenerError::InvalidUrl
    })?;
    Ok(())
}


async fn shorten(&self, url: &str) -> Result<ShortenRes> {
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

### 04 增加单元测试和集成测试

### 05 命令行参数
