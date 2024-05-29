use std::fmt;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use http::{header::LOCATION, HeaderMap, StatusCode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
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
) -> Result<impl IntoResponse, StatusCode> {
    let id = state.shorten(&data.url).await.map_err(|e| {
        warn!("Failed to shorten URL: {e}");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    let body = Json(ShortenRes {
        short_url: format!("http://{}/{}", LISTEN_ADDR, id.short_url),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let url = state
        .get_url(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let db = PgPool::connect(url).await?;

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
        let id = nanoid!(6);
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
            .await?;

        Ok(ShortenReq { url: ret.url })
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
