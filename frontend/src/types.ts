export interface RecipeStage {
  name: string;
  duration_seconds: number;
  target_build_plate_temperature_c: number;
  target_chamber_oxygen_ppm: number;
  target_recoater_position_mm: number;
  tolerance_temperature_c: number;
  tolerance_oxygen_ppm: number;
  description: string;
}

export interface Recipe {
  id: string;
  name: string;
  description: string;
  stages: RecipeStage[];
}

export interface Telemetry {
  run_id: string;
  timestamp: string;
  stage: string;
  elapsed_seconds: number;
  build_plate_temperature_c: number;
  target_build_plate_temperature_c: number;
  chamber_oxygen_ppm: number;
  target_chamber_oxygen_ppm: number;
  recoater_vibration_mm_s: number;
  recoater_position_mm: number;
  target_recoater_position_mm: number;
  scan_track_error_um: number;
  thermal_controller_output: number;
  laser_power_pct: number;
  spatter_rate_per_s: number;
  mean_spatter_velocity_m_s: number;
  mean_spatter_diameter_um: number;
  spatter_angle_deg: number;
  quality_score: number;
  status: RunStatus;
}

export interface StoredTelemetry extends Telemetry {
  id: number;
}

export interface Alert {
  id: string;
  run_id: string;
  timestamp: string;
  severity: "info" | "warning" | "critical";
  code: string;
  message: string;
  stage: string;
}

export interface RunState {
  run_id: string;
  status: RunStatus;
  stage: string;
  stage_progress: number;
  overall_progress: number;
}

export type RunStatus =
  | "idle"
  | "running"
  | "warning"
  | "faulted"
  | "complete"
  | "aborted";

export type ServerMessage =
  | { type: "telemetry"; data: Telemetry }
  | { type: "alert"; data: Alert }
  | { type: "run_state"; data: RunState };

export interface RunSummary {
  id: string;
  recipe_name: string;
  started_at: string;
  ended_at: string | null;
  status: RunStatus;
  duration_seconds: number | null;
  max_build_plate_temperature_c: number | null;
  min_chamber_oxygen_ppm: number | null;
  max_spatter_rate_per_s: number | null;
  alert_count: number;
  final_quality_score: number | null;
}

export interface RunDetail {
  run: RunSummary;
  alerts: Alert[];
}

export interface Metrics {
  active_run: boolean;
  total_runs: number;
  total_alerts: number;
  uptime_seconds: number;
}
