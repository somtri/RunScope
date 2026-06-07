use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::simulator::{pid::PIDController, recipe::RecipeStage};

#[derive(Debug, Clone)]
pub struct TelemetryState {
    pub build_plate_temperature_c: f64,
    pub chamber_oxygen_ppm: f64,
    pub recoater_vibration_mm_s: f64,
    pub recoater_position_mm: f64,
    pub scan_track_error_um: f64,
    pub thermal_controller_output: f64,
    pub laser_power_pct: f64,
    pub spatter_rate_per_s: f64,
    pub mean_spatter_velocity_m_s: f64,
    pub mean_spatter_diameter_um: f64,
    pub spatter_angle_deg: f64,
    pub quality_score: f64,
    rng: StdRng,
}

impl TelemetryState {
    pub fn new(seed: u64) -> Self {
        Self {
            build_plate_temperature_c: 25.0,
            chamber_oxygen_ppm: 1200.0,
            recoater_vibration_mm_s: 0.12,
            recoater_position_mm: 0.0,
            scan_track_error_um: 1.0,
            thermal_controller_output: 0.0,
            laser_power_pct: 0.0,
            spatter_rate_per_s: 0.0,
            mean_spatter_velocity_m_s: 0.0,
            mean_spatter_diameter_um: 0.0,
            spatter_angle_deg: 0.0,
            quality_score: 99.8,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn update(
        &mut self,
        stage: &RecipeStage,
        stage_elapsed: f64,
        total_elapsed: f64,
        dt: f64,
        pid: &mut PIDController,
    ) {
        let noise = self.rng.random_range(-1.0..1.0);
        self.thermal_controller_output = pid.update(
            stage.target_build_plate_temperature_c,
            self.build_plate_temperature_c,
            dt,
        );

        let thermal_rate = if self.thermal_controller_output >= 0.0 {
            13.0
        } else {
            8.5
        };
        self.build_plate_temperature_c += self.thermal_controller_output * thermal_rate * dt;
        self.build_plate_temperature_c += noise * 0.08;
        self.build_plate_temperature_c = self.build_plate_temperature_c.clamp(20.0, 215.0);

        let oxygen_response = if stage.name == "Inerting" { 0.48 } else { 0.24 };
        self.chamber_oxygen_ppm +=
            (stage.target_chamber_oxygen_ppm - self.chamber_oxygen_ppm) * oxygen_response * dt;
        self.chamber_oxygen_ppm += noise * self.chamber_oxygen_ppm.max(40.0) * 0.0012;
        self.chamber_oxygen_ppm = self.chamber_oxygen_ppm.clamp(20.0, 1500.0);

        let recoater_response = if stage.name == "PowderSpread" {
            0.42
        } else {
            0.62
        };
        self.recoater_position_mm += (stage.target_recoater_position_mm
            - self.recoater_position_mm)
            * recoater_response
            * dt;
        self.recoater_position_mm += noise * 0.03;
        self.recoater_position_mm = self.recoater_position_mm.clamp(0.0, 100.0);

        let track_baseline = if stage.name == "LaserScan" { 1.35 } else { 0.8 };
        let track_wave = (total_elapsed * 0.31).sin() * 0.55;
        self.scan_track_error_um +=
            ((track_baseline + track_wave) - self.scan_track_error_um) * 0.32 * dt;
        self.scan_track_error_um += noise * 0.04;
        self.scan_track_error_um = self.scan_track_error_um.clamp(0.1, 8.0);

        let recoater_pulse = stage.name == "PowderSpread" && (stage_elapsed - 6.0).abs() < 0.65;
        let vibration_baseline = if stage.name == "PowderSpread" {
            0.62
        } else {
            0.16
        };
        self.recoater_vibration_mm_s = (vibration_baseline
            + (total_elapsed * 1.7).sin() * 0.06
            + noise * 0.05
            + if recoater_pulse { 2.35 } else { 0.0 })
        .clamp(0.04, 5.0);

        if stage.name == "LaserScan" {
            self.laser_power_pct =
                (72.0 + (stage_elapsed * 0.8).sin() * 4.0 + noise * 2.0).clamp(58.0, 84.0);
            let spatter_pulse = (stage_elapsed - 13.0).abs() < 0.8;
            self.spatter_rate_per_s = (13.0
                + (stage_elapsed * 1.15).sin() * 3.0
                + noise * 2.0
                + if spatter_pulse { 42.0 } else { 0.0 })
            .clamp(2.0, 90.0);
            self.mean_spatter_velocity_m_s =
                (2.8 + self.spatter_rate_per_s * 0.035 + noise * 0.18).clamp(1.5, 7.5);
            self.mean_spatter_diameter_um =
                (48.0 + self.spatter_rate_per_s * 0.45 + noise * 3.0).clamp(30.0, 110.0);
            self.spatter_angle_deg =
                (44.0 + (stage_elapsed * 0.55).sin() * 11.0 + noise * 2.0).clamp(20.0, 75.0);
        } else {
            self.laser_power_pct = 0.0;
            self.spatter_rate_per_s *= 0.55;
            self.mean_spatter_velocity_m_s *= 0.55;
            self.mean_spatter_diameter_um *= 0.55;
            self.spatter_angle_deg *= 0.55;
        }

        let temp_penalty =
            ((self.build_plate_temperature_c - stage.target_build_plate_temperature_c).abs()
                - stage.tolerance_temperature_c)
                .max(0.0)
                * 0.003;
        let oxygen_penalty = ((self.chamber_oxygen_ppm - stage.target_chamber_oxygen_ppm).abs()
            - stage.tolerance_oxygen_ppm)
            .max(0.0)
            * 0.0008;
        let vibration_penalty = (self.recoater_vibration_mm_s - 1.5).max(0.0) * 0.22;
        let spatter_penalty = (self.spatter_rate_per_s - 35.0).max(0.0) * 0.025;
        let track_penalty = (self.scan_track_error_um - 3.0).max(0.0) * 0.04;
        let total_penalty =
            temp_penalty + oxygen_penalty + vibration_penalty + spatter_penalty + track_penalty;
        let recovery = if total_penalty < 0.02 { 0.06 } else { 0.0 };

        self.quality_score += (recovery - total_penalty) * dt;
        self.quality_score = self.quality_score.clamp(72.0, 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::TelemetryState;
    use crate::simulator::{pid::PIDController, recipe::Recipe};

    #[test]
    fn simulator_produces_bounded_continuous_values() {
        let recipe = Recipe::lpbf_layer_demo();
        let stage = &recipe.stages[1];
        let mut telemetry = TelemetryState::new(42);
        let mut pid = PIDController::new(0.018, 0.0007, 0.001, -1.0, 1.0);
        let mut previous = telemetry.build_plate_temperature_c;

        for tick in 0..100 {
            telemetry.update(stage, tick as f64 * 0.3, tick as f64 * 0.3, 0.3, &mut pid);
            assert!((telemetry.build_plate_temperature_c - previous).abs() < 5.0);
            assert!((20.0..=215.0).contains(&telemetry.build_plate_temperature_c));
            assert!((0.04..=5.0).contains(&telemetry.recoater_vibration_mm_s));
            assert!((20.0..=1500.0).contains(&telemetry.chamber_oxygen_ppm));
            assert!((0.0..=100.0).contains(&telemetry.quality_score));
            previous = telemetry.build_plate_temperature_c;
        }
    }
}
