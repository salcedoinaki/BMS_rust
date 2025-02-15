pub struct PidController {
    pub desired_soc: f64,
    kp: f64,
    ki: f64,
    kd: f64,
    last_error: f64,
    integral: f64,
    dt: f64,
}

impl PidController {
    pub fn new(desired_soc: f64, kp: f64, ki: f64, kd: f64, dt: f64) -> Self {
        Self {
            desired_soc,
            kp,
            ki,
            kd,
            last_error: 0.0,
            integral: 0.0,
            dt,
        }
    }

    pub fn compute_load(&mut self, current_soc: f64, disturbance: f64) -> f64 {
        let error = current_soc - self.desired_soc;
        self.integral += error * self.dt;
        let derivative = (error - self.last_error) / self.dt;
        self.last_error = error;
        let control_adjustment = self.kp * error + self.ki * self.integral + self.kd * derivative;
        let load = disturbance + control_adjustment;
        if load < 0.0 { 0.0 } else { load }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_controller_output() {
        let mut pid = PidController::new(70.0, 0.3, 0.05, 0.05, 0.5);
        let load = pid.compute_load(80.0, 0.0);
        // With an error of 10, load should be positive.
        assert!(load > 0.0);
    }
}
