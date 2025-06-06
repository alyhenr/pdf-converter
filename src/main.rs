use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;
use tracing_subscriber;
use tokio::net::TcpListener;

mod routes;
use routes::pdf::upload_pdf_handler;

#[tokio::main]
async fn main() {
    // Enable tracing (logs)
    tracing_subscriber::fmt::init();

    // CORS layer (allow all for now, customize later)
    let cors = CorsLayer::new().allow_origin(Any);

    // Define routes
    let app = Router::new()
        .route("/healthz", get(health_check))
        .route("/convert-pdf", post(upload_pdf_handler))
        .layer(cors);

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 Server running at http://{}", addr);
    axum::serve(TcpListener::bind(&addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}

// Simple GET health check
async fn health_check() -> &'static str {
    "OK"
}
