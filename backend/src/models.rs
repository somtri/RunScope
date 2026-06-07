use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::simulator::recipe::Recipe;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    pub run_id: String,
    pub timestamp: String,
    pub stage: String,
    pub elapsed_seconds: f64,
    pub build_plate_temperature_c: f64,
    pub target_build_plate_temperature_c: f64,
    pub chamber_oxygen_ppm: f64,
    pub target_chamber_oxygen_ppm: f64,
    pub recoater_vibration_mm_s: f64,
    pub recoater_position_mm: f64,
    pub target_recoater_position_mm: f64,
    pub scan_track_error_um: f64,
    pub thermal_controller_output: f64,
    pub laser_power_pct: f64,
    pub spatter_rate_per_s: f64,
    pub mean_spatter_velocity_m_s: f64,
    pub mean_spatter_diameter_um: f64,
    pub spatter_angle_deg: f64,
    pub quality_score: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: String,
    pub run_id: String,
    pub timestamp: String,
    pub severity: String,
    pub code: String,
    pub message: String,
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub run_id: String,
    pub status: String,
    pub stage: String,
    pub stage_progress: f64,
    pub overall_progress: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ServerMessage {
    Telemetry(Telemetry),
    Alert(Alert),
    RunState(RunState),
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RunSummary {
    pub id: String,
    pub recipe_name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub duration_seconds: Option<f64>,
    pub max_build_plate_temperature_c: Option<f64>,
    pub min_chamber_oxygen_ppm: Option<f64>,
    pub max_spatter_rate_per_s: Option<f64>,
    pub alert_count: i64,
    pub final_quality_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct StoredTelemetry {
    pub id: i64,
    pub run_id: String,
    pub timestamp: String,
    pub stage: String,
    pub elapsed_seconds: f64,
    pub build_plate_temperature_c: f64,
    pub target_build_plate_temperature_c: f64,
    pub chamber_oxygen_ppm: f64,
    pub target_chamber_oxygen_ppm: f64,
    pub recoater_vibration_mm_s: f64,
    pub recoater_position_mm: f64,
    pub target_recoater_position_mm: f64,
    pub scan_track_error_um: f64,
    pub thermal_controller_output: f64,
    pub laser_power_pct: f64,
    pub spatter_rate_per_s: f64,
    pub mean_spatter_velocity_m_s: f64,
    pub mean_spatter_diameter_um: f64,
    pub spatter_angle_deg: f64,
    pub quality_score: f64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct StartRunRequest {
    pub recipe_id: String,
}

#[derive(Debug, Serialize)]
pub struct StartRunResponse {
    pub run_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub active_run: bool,
    pub total_runs: i64,
    pub total_alerts: i64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct RunDetail {
    pub run: RunSummary,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Serialize)]
pub struct RecipeList {
    pub recipes: Vec<Recipe>,
}

#[derive(Debug, Serialize)]
pub struct ApiMessage {
    pub message: String,
}
