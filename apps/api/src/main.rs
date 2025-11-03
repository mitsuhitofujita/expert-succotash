use api::{create_router, error::Result, init_db_pool, store::TodoStore};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize tracing
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init_tracing();

    tracing::info!("Starting API server");

    // Initialize database connection pool
    let _db_pool = init_db_pool().await.map_err(|e| {
        tracing::error!("Failed to initialize database connection pool: {e}");
        std::io::Error::other(format!("Database connection failed: {e}"))
    })?;

    tracing::info!("Database connection pool established");
    // NOTE: db_pool will be used as application state in future tasks
    // For now, we continue using TodoStore (in-memory)

    // Initialize data store (will be replaced with database in future)
    let store = TodoStore::new();

    // Create router
    let app = create_router(store);

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
