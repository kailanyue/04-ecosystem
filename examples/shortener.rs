use std::fmt;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
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

const LISTEN_ADDR: &str = "0.0.0.0:9876";

#[derive(Error, Debug)]
pub enum ShortenerError {
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("URL not found")]
    NotFound,
    #[error("Invalid URL")]
    InvalidUrl,
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
            ShortenerError::InvalidUrl => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Failed to shorten URL.")
            } // Handle more error variants as needed.
        };
        let body = Json({
            let mut map = std::collections::HashMap::new();
            map.insert("error", error_message);
            map
        });
        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let url = "postgres://postgres:postgres@192.168.56.101:5432/shortener";

    let state = AppState::try_new(url).await?;
    info!("Connected to database: {url}");

    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("Listening on: {LISTEN_ADDR}");

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, ShortenerError> {
    let id = state.shorten(&data.url).await.map_err(|e| {
        warn!("Failed to shorten URL: {e}");
        ShortenerError::InvalidUrl
    })?;
    let body = Json(ShortenRes {
        short_url: format!("http://{}/{}", LISTEN_ADDR, id.short_url),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ShortenerError> {
    let url = state.get_url(&id).await.map_err(|e| {
        warn!("Failed to get URL: {e}");
        ShortenerError::NotFound
    })?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let db = PgPoolOptions::new().max_connections(5).connect(url).await?;
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

    async fn shorten(&self, url: &str) -> Result<ShortenRes> {
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

    async fn get_url(&self, id: &str) -> Result<ShortenReq> {
        let ret: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await
            .map_err(ShortenerError::DatabaseError)?;

        Ok(ShortenReq { url: ret.url })
    }
}

fn validate_url(url: &str) -> Result<(), ShortenerError> {
    url::Url::parse(url).map_err(|_| {
        warn!("Failed to parse URL: {url}");
        ShortenerError::InvalidUrl
    })?;
    Ok(())
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
