use chrono::Utc;

use crate::{
    models::{Alert, RunState, Telemetry},
    simulator::{
        anomaly::AnomalyDetector,
        pid::PIDController,
        recipe::{Recipe, RecipeSequencer},
        telemetry::TelemetryState,
    },
};

pub struct Simulator {
    run_id: String,
    sequencer: RecipeSequencer,
    telemetry: TelemetryState,
    pid: PIDController,
    anomaly: AnomalyDetector,
    last_stage: Option<String>,
}

pub struct TickResult {
    pub telemetry: Telemetry,
    pub alerts: Vec<Alert>,
    pub run_state: RunState,
    pub complete: bool,
}

impl Simulator {
    pub fn new(run_id: String, recipe: Recipe) -> Self {
        let seed = run_id.bytes().fold(0_u64, |acc, byte| {
            acc.wrapping_mul(31).wrapping_add(byte as u64)
        });
        Self {
            run_id,
            sequencer: RecipeSequencer::new(recipe),
            telemetry: TelemetryState::new(seed),
            pid: PIDController::new(0.018, 0.0007, 0.0015, -1.0, 1.0),
            anomaly: AnomalyDetector::default(),
            last_stage: None,
        }
    }

    pub fn tick(&mut self, dt: f64) -> TickResult {
        if let Some(stage) = self.sequencer.current().cloned() {
            if self.last_stage.as_deref() != Some(stage.name.as_str()) {
                self.pid.reset();
                self.last_stage = Some(stage.name.clone());
            }

            self.telemetry.update(
                &stage,
                self.sequencer.stage_elapsed(),
                self.sequencer.total_elapsed(),
                dt,
                &mut self.pid,
            );
            let alerts = self
                .anomaly
                .evaluate(&self.run_id, &stage, &self.telemetry, dt);
            self.sequencer.update(dt);

            let complete = self.sequencer.is_complete();
            let status = if complete { "complete" } else { "running" };
            let next_stage = self
                .sequencer
                .current()
                .map(|value| value.name.clone())
                .unwrap_or_else(|| "Complete".into());

            let telemetry = Telemetry {
                run_id: self.run_id.clone(),
                timestamp: Utc::now().to_rfc3339(),
                stage: if complete {
                    "Complete".into()
                } else {
                    stage.name
                },
                elapsed_seconds: self.sequencer.total_elapsed(),
                build_plate_temperature_c: round(self.telemetry.build_plate_temperature_c),
                target_build_plate_temperature_c: round(stage.target_build_plate_temperature_c),
                chamber_oxygen_ppm: round(self.telemetry.chamber_oxygen_ppm),
                target_chamber_oxygen_ppm: round(stage.target_chamber_oxygen_ppm),
                recoater_vibration_mm_s: round(self.telemetry.recoater_vibration_mm_s),
                recoater_position_mm: round(self.telemetry.recoater_position_mm),
                target_recoater_position_mm: round(stage.target_recoater_position_mm),
                scan_track_error_um: round(self.telemetry.scan_track_error_um),
                thermal_controller_output: round(self.telemetry.thermal_controller_output),
                laser_power_pct: round(self.telemetry.laser_power_pct),
                spatter_rate_per_s: round(self.telemetry.spatter_rate_per_s),
                mean_spatter_velocity_m_s: round(self.telemetry.mean_spatter_velocity_m_s),
                mean_spatter_diameter_um: round(self.telemetry.mean_spatter_diameter_um),
                spatter_angle_deg: round(self.telemetry.spatter_angle_deg),
                quality_score: round(self.telemetry.quality_score),
                status: status.into(),
            };

            TickResult {
                telemetry,
                alerts,
                run_state: RunState {
                    run_id: self.run_id.clone(),
                    status: status.into(),
                    stage: next_stage,
                    stage_progress: self.sequencer.stage_progress(),
                    overall_progress: self.sequencer.overall_progress(),
                },
                complete,
            }
        } else {
            unreachable!("simulator tick called after recipe completion")
        }
    }
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
