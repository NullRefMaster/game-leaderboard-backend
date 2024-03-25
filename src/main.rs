use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{convert::Infallible, env, time::Duration};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

// Define data structures for requests and responses
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

// Define handlers for API endpoints
async fn get_leaderboard(
    State(pool): State<PgPool>,
    Path(level): Path<i32>,
) -> Result<impl IntoResponse, Infallible> {
    let entries = sqlx::query_as!(
        LeaderboardEntry,
        "SELECT ROW_NUMBER() OVER (ORDER BY time) as rank, player, time FROM leaderboard WHERE level = $1",
        level
    );

    Ok(axum::Json(entries.fetch_all(&pool).await.unwrap()))
}

async fn upload_data(
    State(pool): State<PgPool>,
    Path(level): Path<i32>,
    Json(payload): Json<UploadRequest>,
) -> (StatusCode, String) {
    // Store uploaded data in the database
    let result = sqlx::query!(
        "INSERT INTO leaderboard (level, player, time) VALUES ($1, $2, $3)",
        level,
        payload.player,
        payload.time
    );

    match result.execute(&pool).await {
        Ok(_) => (
            StatusCode::CREATED,
            "Data uploaded successfully".to_string(),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to upload data".to_string(),
        ),
    }
}

#[tokio::main]
async fn main() {
    // Load database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set in environment");

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
                .allow_methods([Method::GET, Method::POST]),
        )
        .with_state(pool);

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
