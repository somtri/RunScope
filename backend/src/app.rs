use std::{sync::Arc, time::Instant};

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, watch, RwLock};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    api,
    models::{RunState, ServerMessage},
    simulator::recipe::Recipe,
    ws,
};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub started_at: Instant,
    pub recipes: Arc<Vec<Recipe>>,
    pub active_run: Arc<RwLock<Option<ActiveRun>>>,
    pub broadcaster: broadcast::Sender<ServerMessage>,
}

pub struct ActiveRun {
    pub state: RunState,
    pub elapsed_seconds: f64,
    pub abort_tx: watch::Sender<bool>,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        let (broadcaster, _) = broadcast::channel(512);
        Self {
            db,
            started_at: Instant::now(),
            recipes: Arc::new(vec![Recipe::lpbf_layer_demo()]),
            active_run: Arc::new(RwLock::new(None)),
            broadcaster,
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(api::health))
        .route("/api/recipes", get(api::recipes))
        .route("/api/runs/start", post(api::start_run))
        .route("/api/runs/{run_id}/abort", post(api::abort_run))
        .route("/api/runs", get(api::runs))
        .route("/api/runs/{run_id}", get(api::run_detail))
        .route("/api/runs/{run_id}/telemetry", get(api::run_telemetry))
        .route("/api/metrics", get(api::metrics))
        .route("/ws/telemetry", get(ws::telemetry_socket))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
