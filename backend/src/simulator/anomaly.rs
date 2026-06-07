use chrono::Utc;
use uuid::Uuid;

use crate::{
    models::Alert,
    simulator::{recipe::RecipeStage, telemetry::TelemetryState},
};

#[derive(Debug, Default)]
pub struct AnomalyDetector {
    current_stage: Option<String>,
    temperature_out_seconds: f64,
    oxygen_out_seconds: f64,
    saturation_seconds: f64,
    recoater_latched: bool,
    alignment_latched: bool,
    spatter_latched: bool,
    quality_latched: bool,
}

impl AnomalyDetector {
    pub fn evaluate(
        &mut self,
        run_id: &str,
        stage: &RecipeStage,
        telemetry: &TelemetryState,
        dt: f64,
    ) -> Vec<Alert> {
        if self.current_stage.as_deref() != Some(stage.name.as_str()) {
            self.current_stage = Some(stage.name.clone());
            self.temperature_out_seconds = 0.0;
            self.oxygen_out_seconds = 0.0;
            self.saturation_seconds = 0.0;
        }

        let mut alerts = Vec::new();
        let temperature_error =
            (telemetry.build_plate_temperature_c - stage.target_build_plate_temperature_c).abs();
        let oxygen_error = (telemetry.chamber_oxygen_ppm - stage.target_chamber_oxygen_ppm).abs();
        let settling_stage = matches!(stage.name.as_str(), "Inerting" | "Preheat" | "Cooldown");

        self.temperature_out_seconds = accumulate(
            self.temperature_out_seconds,
            temperature_error > stage.tolerance_temperature_c,
            dt,
        );
        self.oxygen_out_seconds = accumulate(
            self.oxygen_out_seconds,
            oxygen_error > stage.tolerance_oxygen_ppm,
            dt,
        );
        self.saturation_seconds = accumulate(
            self.saturation_seconds,
            telemetry.thermal_controller_output.abs() > 0.98,
            dt,
        );

        if !settling_stage && self.temperature_out_seconds >= 4.0 {
            let severity = if temperature_error > stage.tolerance_temperature_c * 4.0 {
                "critical"
            } else {
                "warning"
            };
            alerts.push(alert(
                run_id,
                severity,
                "PLATE_TEMP_OUT_OF_RANGE",
                format!(
                    "Build-plate temperature is {:.1} C from target during {}",
                    temperature_error, stage.name
                ),
                &stage.name,
            ));
            self.temperature_out_seconds = -8.0;
        }

        if !settling_stage && self.oxygen_out_seconds >= 4.0 {
            let severity = if telemetry.chamber_oxygen_ppm > 250.0 {
                "critical"
            } else {
                "warning"
            };
            alerts.push(alert(
                run_id,
                severity,
                "OXYGEN_DRIFT",
                format!(
                    "Chamber oxygen remains {:.0} ppm outside its target",
                    oxygen_error
                ),
                &stage.name,
            ));
            self.oxygen_out_seconds = -8.0;
        }

        if stage.name == "PowderSpread"
            && telemetry.recoater_vibration_mm_s > 2.4
            && !self.recoater_latched
        {
            let severity = if telemetry.recoater_vibration_mm_s > 4.0 {
                "critical"
            } else {
                "warning"
            };
            alerts.push(alert(
                run_id,
                severity,
                "RECOATER_VIBRATION",
                format!(
                    "Recoater vibration reached {:.2} mm/s during powder spreading",
                    telemetry.recoater_vibration_mm_s
                ),
                &stage.name,
            ));
            self.recoater_latched = true;
        } else if telemetry.recoater_vibration_mm_s < 1.2 {
            self.recoater_latched = false;
        }

        if stage.name == "LaserScan"
            && telemetry.scan_track_error_um > 4.5
            && !self.alignment_latched
        {
            alerts.push(alert(
                run_id,
                "warning",
                "TRACK_ALIGNMENT_ERROR",
                format!(
                    "Scan-track alignment error reached {:.2} um",
                    telemetry.scan_track_error_um
                ),
                &stage.name,
            ));
            self.alignment_latched = true;
        } else if telemetry.scan_track_error_um < 3.0 {
            self.alignment_latched = false;
        }

        if stage.name == "LaserScan" && telemetry.spatter_rate_per_s > 45.0 && !self.spatter_latched
        {
            let severity = if telemetry.spatter_rate_per_s > 80.0 {
                "critical"
            } else {
                "warning"
            };
            alerts.push(alert(
                run_id,
                severity,
                "SPATTER_BURST",
                format!(
                    "Detected {:.1} spatter ejections/s with {:.1} m/s mean velocity",
                    telemetry.spatter_rate_per_s, telemetry.mean_spatter_velocity_m_s
                ),
                &stage.name,
            ));
            self.spatter_latched = true;
        } else if telemetry.spatter_rate_per_s < 25.0 {
            self.spatter_latched = false;
        }

        if !settling_stage && self.saturation_seconds >= 8.0 {
            alerts.push(alert(
                run_id,
                "warning",
                "CONTROL_SATURATION",
                "Temperature controller has remained saturated".into(),
                &stage.name,
            ));
            self.saturation_seconds = -10.0;
        }

        if telemetry.quality_score < 82.0 && !self.quality_latched {
            alerts.push(alert(
                run_id,
                "critical",
                "QUALITY_DROP",
                format!(
                    "Run quality dropped to {:.1} percent",
                    telemetry.quality_score
                ),
                &stage.name,
            ));
            self.quality_latched = true;
        }

        alerts
    }
}

fn accumulate(current: f64, active: bool, dt: f64) -> f64 {
    if active {
        current + dt
    } else {
        current.max(0.0) * 0.5
    }
}

fn alert(run_id: &str, severity: &str, code: &str, message: String, stage: &str) -> Alert {
    Alert {
        id: Uuid::new_v4().to_string(),
        run_id: run_id.into(),
        timestamp: Utc::now().to_rfc3339(),
        severity: severity.into(),
        code: code.into(),
        message,
        stage: stage.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::AnomalyDetector;
    use crate::simulator::{recipe::Recipe, telemetry::TelemetryState};

    #[test]
    fn high_recoater_vibration_triggers_warning() {
        let recipe = Recipe::lpbf_layer_demo();
        let stage = &recipe.stages[2];
        let mut detector = AnomalyDetector::default();
        let mut telemetry = TelemetryState::new(1);
        telemetry.recoater_vibration_mm_s = 3.1;

        let alerts = detector.evaluate("test-run", stage, &telemetry, 0.3);
        assert!(alerts
            .iter()
            .any(|alert| alert.code == "RECOATER_VIBRATION" && alert.severity == "warning"));
    }

    #[test]
    fn spatter_burst_triggers_warning() {
        let recipe = Recipe::lpbf_layer_demo();
        let stage = &recipe.stages[3];
        let mut detector = AnomalyDetector::default();
        let mut telemetry = TelemetryState::new(1);
        telemetry.spatter_rate_per_s = 55.0;
        telemetry.mean_spatter_velocity_m_s = 4.7;

        let alerts = detector.evaluate("test-run", stage, &telemetry, 0.3);
        assert!(alerts
            .iter()
            .any(|alert| alert.code == "SPATTER_BURST" && alert.severity == "warning"));
    }

    #[test]
    fn extreme_temperature_deviation_triggers_critical() {
        let recipe = Recipe::lpbf_layer_demo();
        let stage = &recipe.stages[3];
        let mut detector = AnomalyDetector::default();
        let mut telemetry = TelemetryState::new(1);
        telemetry.build_plate_temperature_c = stage.target_build_plate_temperature_c - 30.0;

        let mut alerts = Vec::new();
        for _ in 0..15 {
            alerts.extend(detector.evaluate("test-run", stage, &telemetry, 0.3));
        }

        assert!(alerts
            .iter()
            .any(|alert| alert.code == "PLATE_TEMP_OUT_OF_RANGE" && alert.severity == "critical"));
    }

    #[test]
    fn stage_change_clears_out_of_range_timers() {
        let recipe = Recipe::lpbf_layer_demo();
        let preheat = &recipe.stages[1];
        let powder_spread = &recipe.stages[2];
        let mut detector = AnomalyDetector::default();
        let mut telemetry = TelemetryState::new(1);
        telemetry.build_plate_temperature_c = preheat.target_build_plate_temperature_c - 50.0;

        for _ in 0..20 {
            let _ = detector.evaluate("test-run", preheat, &telemetry, 0.3);
        }

        telemetry.build_plate_temperature_c = powder_spread.target_build_plate_temperature_c;
        telemetry.chamber_oxygen_ppm = powder_spread.target_chamber_oxygen_ppm;
        telemetry.thermal_controller_output = 0.0;
        let alerts = detector.evaluate("test-run", powder_spread, &telemetry, 0.3);

        assert!(alerts.is_empty());
    }
}
