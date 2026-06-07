mod api;
mod app;
mod db;
mod models;
mod simulator;
mod ws;

use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "run_scope_backend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = db::connect("sqlite://run_scope.db").await?;
    let state = app::AppState::new(pool);
    let app = app::router(state);
    let address = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(address).await?;

    tracing::info!("run_scope backend listening on http://{address}");
    axum::serve(listener, app).await?;
    Ok(())
}
