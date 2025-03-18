/// Basic PID Controller
pub struct PidController {
    kp: f64,
    ki: f64,
    kd: f64,
    last_error: f64,
    integral: f64,
    dt: f64,
}

impl PidController {
    /// Creates a new PID controller with gains and dt
    pub fn new(kp: f64, ki: f64, kd: f64, dt: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            last_error: 0.0,
            integral: 0.0,
            dt,
        }
    }

    /// Compute control signal based on setpoint vs measured
    pub fn compute(&mut self, setpoint: f64, measured: f64) -> f64 {
        let error = setpoint - measured;
        self.integral += error * self.dt;
        let derivative = (error - self.last_error) / self.dt;
        self.last_error = error;
        // PID output
        self.kp * error + self.ki * self.integral + self.kd * derivative
    }
}

/// High-level oxygen regulator built on top of PID
pub struct OxygenController {
    pid: PidController,
}

impl OxygenController {
    pub fn new(kp: f64, ki: f64, kd: f64, dt: f64) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd, dt),
        }
    }

    pub fn regulate(&mut self, desired: f64, measured: f64) -> f64 {
        self.pid.compute(desired, measured)
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
