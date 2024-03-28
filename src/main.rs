use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

#[derive(Debug, Serialize, Deserialize)]
struct LeaderboardEntry {
    rank: Option<i64>,
    player: String,
    time: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadRequest {
    player: String,
    time: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadResponse {
    success: bool,
    message: String,
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    key: Arc<Vec<u8>>,
}

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set in environment");
    let key = env::var("KEY").expect("KEY not set in environment");
    let key = hex::decode(key).expect("Failed to decode key");
    let key = Arc::new(key);

    // Create a database pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Create an Axum application with routes
    let app = Router::new()
        .route(
            "/leaderboard/:level",
            get(get_leaderboard).post(upload_data),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::any())
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(Any),
        )
        .with_state(AppState { pool, key });

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_leaderboard(
    State(state): State<AppState>,
    Path(level): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let entries = sqlx::query_as!(
        LeaderboardEntry,
        "SELECT ROW_NUMBER() OVER (ORDER BY time) as rank, player, time FROM leaderboard WHERE level = $1 ORDER BY time LIMIT 100",
        level
    );

    let data = entries
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::Json(data))
}

type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

async fn upload_data(
    State(state): State<AppState>,
    Path(level): Path<i32>,
    body: Bytes,
) -> Result<(), StatusCode> {
    let decrypted_body = decrypt(&state.key, &body).ok_or(StatusCode::BAD_REQUEST)?;
    let payload = serde_json::from_slice::<UploadRequest>(&decrypted_body)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Store uploaded data in the database
    let result = sqlx::query!(
        "INSERT INTO leaderboard (level, player, time) VALUES ($1, $2, $3)",
        level,
        payload.player,
        payload.time
    );

    result
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

fn decrypt(key: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    data.len().checked_sub(16)?;
    let iv = &data[..16];
    let ciphertext = &data[16..];
    let res = Aes256CbcDec::new(key.into(), iv.into()).decrypt_padded_vec_mut::<Pkcs7>(ciphertext);
    res.ok()
}
