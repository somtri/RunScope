use std::{convert::Infallible, time::Duration};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Serialize;
use sqlx::Row;
use tokio::{sync::watch, time::MissedTickBehavior};
use uuid::Uuid;

use crate::{
    app::{ActiveRun, AppState},
    models::{
        Alert, ApiMessage, HealthResponse, MetricsResponse, RecipeList, RunDetail, RunState,
        RunSummary, ServerMessage, StartRunRequest, StartRunResponse, StoredTelemetry, Telemetry,
    },
    simulator::{engine::Simulator, recipe::Recipe},
};

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "run_scope-backend",
        uptime_seconds: state.uptime_seconds(),
    })
}

pub async fn recipes(State(state): State<AppState>) -> Json<RecipeList> {
    Json(RecipeList {
        recipes: state.recipes.as_ref().clone(),
    })
}

pub async fn start_run(
    State(state): State<AppState>,
    Json(request): Json<StartRunRequest>,
) -> Result<(StatusCode, Json<StartRunResponse>), ApiError> {
    let recipe = state
        .recipes
        .iter()
        .find(|recipe| recipe.id == request.recipe_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("Recipe not found"))?;

    let mut active = state.active_run.write().await;
    if active
        .as_ref()
        .is_some_and(|run| run.state.status == "running")
    {
        return Err(ApiError::conflict("A run is already active"));
    }

    let run_id = Uuid::new_v4().to_string();
    let started_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO runs (id, recipe_name, started_at, status) VALUES (?, ?, ?, 'running')",
    )
    .bind(&run_id)
    .bind(&recipe.name)
    .bind(started_at)
    .execute(&state.db)
    .await?;

    let (abort_tx, abort_rx) = watch::channel(false);
    let initial_state = RunState {
        run_id: run_id.clone(),
        status: "running".into(),
        stage: recipe
            .stages
            .first()
            .map(|stage| stage.name.clone())
            .unwrap_or_else(|| "Idle".into()),
        stage_progress: 0.0,
        overall_progress: 0.0,
    };
    *active = Some(ActiveRun {
        state: initial_state.clone(),
        elapsed_seconds: 0.0,
        abort_tx,
    });
    drop(active);

    let _ = state
        .broadcaster
        .send(ServerMessage::RunState(initial_state));
    tokio::spawn(run_engine(state, run_id.clone(), recipe, abort_rx));

    Ok((
        StatusCode::CREATED,
        Json(StartRunResponse {
            run_id,
            status: "running".into(),
        }),
    ))
}

pub async fn abort_run(
    Path(run_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiMessage>, ApiError> {
    let active = state.active_run.read().await;
    let run = active
        .as_ref()
        .filter(|run| run.state.run_id == run_id)
        .ok_or_else(|| ApiError::not_found("Active run not found"))?;
    run.abort_tx
        .send(true)
        .map_err(|_| ApiError::internal("Run control channel is closed"))?;

    Ok(Json(ApiMessage {
        message: "Abort requested".into(),
    }))
}

pub async fn runs(State(state): State<AppState>) -> Result<Json<Vec<RunSummary>>, ApiError> {
    let runs = sqlx::query_as::<_, RunSummary>(
        "SELECT id, recipe_name, started_at, ended_at, status, duration_seconds,
         max_build_plate_temperature_c, min_chamber_oxygen_ppm, max_spatter_rate_per_s, alert_count,
         final_quality_score FROM runs ORDER BY started_at DESC",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(runs))
}

pub async fn run_detail(
    Path(run_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<RunDetail>, ApiError> {
    let run = sqlx::query_as::<_, RunSummary>(
        "SELECT id, recipe_name, started_at, ended_at, status, duration_seconds,
         max_build_plate_temperature_c, min_chamber_oxygen_ppm, max_spatter_rate_per_s, alert_count,
         final_quality_score FROM runs WHERE id = ?",
    )
    .bind(&run_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Run not found"))?;

    let alerts = sqlx::query_as::<_, Alert>(
        "SELECT id, run_id, timestamp, severity, code, message, stage
         FROM alerts WHERE run_id = ? ORDER BY timestamp ASC",
    )
    .bind(run_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(RunDetail { run, alerts }))
}

pub async fn run_telemetry(
    Path(run_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StoredTelemetry>>, ApiError> {
    let samples = sqlx::query_as::<_, StoredTelemetry>(
        "SELECT id, run_id, timestamp, stage, elapsed_seconds, build_plate_temperature_c,
         target_build_plate_temperature_c, chamber_oxygen_ppm, target_chamber_oxygen_ppm,
         recoater_vibration_mm_s, recoater_position_mm, target_recoater_position_mm,
         scan_track_error_um, thermal_controller_output, laser_power_pct, spatter_rate_per_s,
         mean_spatter_velocity_m_s, mean_spatter_diameter_um, spatter_angle_deg,
         quality_score, status
         FROM telemetry_samples WHERE run_id = ? ORDER BY timestamp ASC",
    )
    .bind(run_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(samples))
}

pub async fn metrics(State(state): State<AppState>) -> Result<Json<MetricsResponse>, ApiError> {
    let run_count = sqlx::query("SELECT COUNT(*) AS count FROM runs")
        .fetch_one(&state.db)
        .await?
        .get::<i64, _>("count");
    let alert_count = sqlx::query("SELECT COUNT(*) AS count FROM alerts")
        .fetch_one(&state.db)
        .await?
        .get::<i64, _>("count");
    let active_run = state.active_run.read().await.is_some();

    Ok(Json(MetricsResponse {
        active_run,
        total_runs: run_count,
        total_alerts: alert_count,
        uptime_seconds: state.uptime_seconds(),
    }))
}

async fn run_engine(
    state: AppState,
    run_id: String,
    recipe: Recipe,
    mut abort_rx: watch::Receiver<bool>,
) {
    let mut simulator = Simulator::new(run_id.clone(), recipe);
    let mut interval = tokio::time::interval(Duration::from_millis(300));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_saved = -1.0_f64;
    let mut max_build_plate_temperature = f64::MIN;
    let mut min_chamber_oxygen = f64::MAX;
    let mut max_spatter_rate = f64::MIN;
    let mut alert_count = 0_i64;
    let mut final_quality = 100.0;
    let final_status;
    let final_elapsed;

    loop {
        tokio::select! {
            _ = interval.tick() => {}
            changed = abort_rx.changed() => {
                if changed.is_ok() && *abort_rx.borrow() {
                    final_status = "aborted".to_string();
                    let (stage_progress, overall_progress, elapsed_seconds) =
                        active_progress(&state, &run_id).await;
                    final_elapsed = elapsed_seconds;
                    let state_message = RunState {
                        run_id: run_id.clone(),
                        status: final_status.clone(),
                        stage: "Aborted".into(),
                        stage_progress,
                        overall_progress,
                    };
                    update_active_state(
                        &state,
                        state_message.clone(),
                        final_elapsed,
                    )
                    .await;
                    let _ = state.broadcaster.send(ServerMessage::RunState(state_message));
                    break;
                }
                continue;
            }
        }

        let tick = simulator.tick(0.3);
        max_build_plate_temperature =
            max_build_plate_temperature.max(tick.telemetry.build_plate_temperature_c);
        min_chamber_oxygen = min_chamber_oxygen.min(tick.telemetry.chamber_oxygen_ppm);
        max_spatter_rate = max_spatter_rate.max(tick.telemetry.spatter_rate_per_s);
        final_quality = tick.telemetry.quality_score;

        update_active_state(
            &state,
            tick.run_state.clone(),
            tick.telemetry.elapsed_seconds,
        )
        .await;
        let _ = state
            .broadcaster
            .send(ServerMessage::Telemetry(tick.telemetry.clone()));
        let _ = state
            .broadcaster
            .send(ServerMessage::RunState(tick.run_state.clone()));

        for alert in &tick.alerts {
            alert_count += 1;
            if let Err(error) = save_alert(&state, alert).await {
                tracing::error!(%error, "failed to save alert");
            }
            let _ = state.broadcaster.send(ServerMessage::Alert(alert.clone()));
        }

        if tick.telemetry.elapsed_seconds - last_saved >= 1.0 || tick.complete {
            if let Err(error) = save_telemetry(&state, &tick.telemetry).await {
                tracing::error!(%error, "failed to save telemetry");
            }
            last_saved = tick.telemetry.elapsed_seconds;
        }

        let critical = tick.alerts.iter().any(|alert| alert.severity == "critical");
        if critical || tick.complete {
            final_status = if critical { "faulted" } else { "complete" }.into();
            final_elapsed = tick.telemetry.elapsed_seconds;
            if critical {
                let fault_state = RunState {
                    run_id: run_id.clone(),
                    status: "faulted".into(),
                    stage: "Faulted".into(),
                    stage_progress: 1.0,
                    overall_progress: tick.run_state.overall_progress,
                };
                update_active_state(&state, fault_state.clone(), tick.telemetry.elapsed_seconds)
                    .await;
                let _ = state.broadcaster.send(ServerMessage::RunState(fault_state));
            }
            break;
        }
    }

    if let Err(error) = finalize_run(
        &state,
        &run_id,
        &final_status,
        final_elapsed,
        max_build_plate_temperature,
        min_chamber_oxygen,
        max_spatter_rate,
        alert_count,
        final_quality,
    )
    .await
    {
        tracing::error!(%error, "failed to finalize run");
    }

    let mut active = state.active_run.write().await;
    if active
        .as_ref()
        .is_some_and(|run| run.state.run_id == run_id)
    {
        *active = None;
    }
}

async fn active_progress(state: &AppState, run_id: &str) -> (f64, f64, f64) {
    let active = state.active_run.read().await;
    active
        .as_ref()
        .filter(|run| run.state.run_id == run_id)
        .map(|run| {
            (
                run.state.stage_progress,
                run.state.overall_progress,
                run.elapsed_seconds,
            )
        })
        .unwrap_or((0.0, 0.0, 0.0))
}

async fn update_active_state(state: &AppState, run_state: RunState, elapsed_seconds: f64) {
    let mut active = state.active_run.write().await;
    if let Some(active) = active.as_mut() {
        if active.state.run_id == run_state.run_id {
            active.state = run_state;
            active.elapsed_seconds = elapsed_seconds;
        }
    }
}

async fn save_telemetry(state: &AppState, telemetry: &Telemetry) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO telemetry_samples (
            run_id, timestamp, stage, elapsed_seconds, build_plate_temperature_c,
            target_build_plate_temperature_c, chamber_oxygen_ppm, target_chamber_oxygen_ppm,
            recoater_vibration_mm_s, recoater_position_mm, target_recoater_position_mm,
            scan_track_error_um, thermal_controller_output, laser_power_pct, spatter_rate_per_s,
            mean_spatter_velocity_m_s, mean_spatter_diameter_um, spatter_angle_deg,
            quality_score, status
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&telemetry.run_id)
    .bind(&telemetry.timestamp)
    .bind(&telemetry.stage)
    .bind(telemetry.elapsed_seconds)
    .bind(telemetry.build_plate_temperature_c)
    .bind(telemetry.target_build_plate_temperature_c)
    .bind(telemetry.chamber_oxygen_ppm)
    .bind(telemetry.target_chamber_oxygen_ppm)
    .bind(telemetry.recoater_vibration_mm_s)
    .bind(telemetry.recoater_position_mm)
    .bind(telemetry.target_recoater_position_mm)
    .bind(telemetry.scan_track_error_um)
    .bind(telemetry.thermal_controller_output)
    .bind(telemetry.laser_power_pct)
    .bind(telemetry.spatter_rate_per_s)
    .bind(telemetry.mean_spatter_velocity_m_s)
    .bind(telemetry.mean_spatter_diameter_um)
    .bind(telemetry.spatter_angle_deg)
    .bind(telemetry.quality_score)
    .bind(&telemetry.status)
    .execute(&state.db)
    .await?;
    Ok(())
}

async fn save_alert(state: &AppState, alert: &Alert) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO alerts (id, run_id, timestamp, severity, code, message, stage)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&alert.id)
    .bind(&alert.run_id)
    .bind(&alert.timestamp)
    .bind(&alert.severity)
    .bind(&alert.code)
    .bind(&alert.message)
    .bind(&alert.stage)
    .execute(&state.db)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn finalize_run(
    state: &AppState,
    run_id: &str,
    status: &str,
    duration: f64,
    max_build_plate_temperature: f64,
    min_chamber_oxygen: f64,
    max_spatter_rate: f64,
    alert_count: i64,
    final_quality: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE runs SET ended_at = ?, status = ?, duration_seconds = ?,
         max_build_plate_temperature_c = ?, min_chamber_oxygen_ppm = ?,
         max_spatter_rate_per_s = ?, alert_count = ?, final_quality_score = ?
         WHERE id = ?",
    )
    .bind(Utc::now().to_rfc3339())
    .bind(status)
    .bind(duration)
    .bind(finite_or_zero(max_build_plate_temperature))
    .bind(finite_or_zero(min_chamber_oxygen))
    .bind(finite_or_zero(max_spatter_rate))
    .bind(alert_count)
    .bind(final_quality)
    .bind(run_id)
    .execute(&state.db)
    .await?;
    Ok(())
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    fn internal(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "database request failed");
        Self::internal("Database request failed")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody {
            error: String,
        }

        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

impl From<Infallible> for ApiError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
