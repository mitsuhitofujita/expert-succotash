use axum::{Json, Router, routing::get};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[tokio::main]
async fn main() {
    // ルーティングの設定
    let app = Router::new().route("/health", get(health_check));

    // サーバーアドレスの設定
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {addr}");

    // サーバーの起動
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
