#[derive(Debug, Clone)]
pub struct PIDController {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub integral: f64,
    pub previous_error: f64,
    pub output_min: f64,
    pub output_max: f64,
}

impl PIDController {
    pub fn new(kp: f64, ki: f64, kd: f64, output_min: f64, output_max: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            previous_error: 0.0,
            output_min,
            output_max,
        }
    }

    pub fn update(&mut self, setpoint: f64, measured: f64, dt: f64) -> f64 {
        if dt <= 0.0 {
            return 0.0;
        }

        let error = setpoint - measured;
        self.integral += error * dt;
        let derivative = (error - self.previous_error) / dt;
        self.previous_error = error;

        let raw = self.kp * error + self.ki * self.integral + self.kd * derivative;
        let output = raw.clamp(self.output_min, self.output_max);

        if raw != output && self.ki.abs() > f64::EPSILON {
            self.integral -= error * dt;
        }

        output
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.previous_error = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::PIDController;

    #[test]
    fn output_increases_when_measurement_is_below_setpoint() {
        let mut pid = PIDController::new(0.02, 0.001, 0.0, -1.0, 1.0);
        let near = pid.update(100.0, 95.0, 1.0);
        pid.reset();
        let far = pid.update(100.0, 70.0, 1.0);
        assert!(far > near);
        assert!(far <= 1.0);
    }
}
