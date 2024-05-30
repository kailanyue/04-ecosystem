use std::fmt;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use http::{header::LOCATION, HeaderMap, StatusCode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Debug, Deserialize, Serialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ShortenRes {
    short_url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    // Pool is an Arc<PoolInner<DB>> , 因此可以使用 Clone
    // pub struct Pool<DB: Database>(pub(crate) Arc<PoolInner<DB>>);
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[derive(Error, Debug)]
pub enum ShortenerError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("URL not found")]
    NotFound,
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

#[derive(Clone, Debug, Parser)]
#[command(name="url", version, author, about, long_about = None)]
struct Config {
    /// 数据库连接字符串
    #[arg(
        long,
        default_value = "postgres://postgres:postgres@192.168.1.9:5432/shortener",
        help = "database url"
    )]
    database_url: String,

    /// 监听地址
    #[arg(long, default_value = "0.0.0.0:9876", help = "listen address")]
    listen_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = Config::parse();

    let database_url = config.database_url;
    let state = AppState::try_new(&database_url).await?;
    info!("Connected to database: {database_url}");

    let listen_addr = config.listen_addr;
    let listener = TcpListener::bind(&listen_addr).await?;
    info!("Listening on: {}", listen_addr);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state((state, listen_addr)); // 将配置传递给应用状态

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn shorten(
    State((state, listen_addr)): State<(AppState, String)>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, ShortenerError> {
    let id = state.shorten(&data.url).await.map_err(|e| {
        warn!("Failed to shorten URL: {e}");
        ShortenerError::InvalidUrl(data.url)
    })?;
    let body = Json(ShortenRes {
        short_url: format!("http://{}/{}", listen_addr, id.short_url),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    Path(id): Path<String>,
    State((state, _)): State<(AppState, String)>,
) -> Result<impl IntoResponse, ShortenerError> {
    let url = state.get_url(&id).await?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(database_url: &str) -> Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        info!("Successfully connected to the database!");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(&db)
        .await?;

        Ok(Self { db })
    }

    async fn shorten(&self, url: &str) -> Result<ShortenRes, ShortenerError> {
        validate_url(url)?;
        let mut id = nanoid!(6);
        loop {
            let ret: Option<UrlRecord> = sqlx::query_as("SELECT id FROM urls WHERE id = $1")
                .bind(&id)
                .fetch_optional(&self.db)
                .await?;
            match ret {
                Some(_) => {
                    warn!("Collision on ID: {id}");
                    id = nanoid!(6);
                }
                None => break,
            }
        }

        let ret:UrlRecord = sqlx::query_as(
                    "INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url=EXCLUDED.url RETURNING id",
                )
                .bind(&id)
                .bind(url)
                .fetch_one(&self.db)
                .await?;

        Ok(ShortenRes { short_url: ret.id })
    }

    async fn get_url(&self, id: &str) -> Result<UrlRecord, ShortenerError> {
        let ret = sqlx::query_as::<_, UrlRecord>("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await?;

        match ret {
            Some(url_record) => Ok(url_record),
            None => Err(ShortenerError::NotFound),
        }
    }
}

fn validate_url(url: &str) -> Result<(), ShortenerError> {
    let parsed_url =
        url::Url::parse(url).map_err(|_| ShortenerError::InvalidUrl(url.to_string()))?;
    if !["http", "https"].contains(&parsed_url.scheme()) {
        return Err(ShortenerError::InvalidUrl(url.to_string()));
    }
    Ok(())
}

// Implement `IntoResponse` for `ShortenerError`.
impl IntoResponse for ShortenerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            ShortenerError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error occurred.",
            ),
            ShortenerError::NotFound => (StatusCode::NOT_FOUND, "URL parsing error occurred."),
            ShortenerError::InvalidUrl(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Failed to shorten URL.")
            }
        };
        let body = Json({
            let mut map = std::collections::HashMap::new();
            map.insert("error", error_message);
            map
        });
        (status, body).into_response()
    }
}

impl fmt::Display for ShortenRes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.short_url)
    }
}

impl fmt::Display for ShortenReq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}
