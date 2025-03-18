/// Basic PID Controller with adaptive gain scheduling capability.
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

    /// Compute control signal based on setpoint vs measured using fixed gains.
    pub fn compute(&mut self, setpoint: f64, measured: f64) -> f64 {
        let error = setpoint - measured;
        self.integral += error * self.dt;
        let derivative = (error - self.last_error) / self.dt;
        self.last_error = error;
        self.kp * error + self.ki * self.integral + self.kd * derivative
    }

    /// Compute control signal using adaptive gain scheduling.
    /// This method increases the proportional gain when the error magnitude is high.
    pub fn compute_adaptive(&mut self, setpoint: f64, measured: f64) -> f64 {
        let error = setpoint - measured;
        let factor = if error.abs() > 1.0 { 1.5 } else { 1.0 };
        factor * self.compute(setpoint, measured)
    }
}

/// High-level oxygen regulator built on top of PID.
pub struct OxygenController {
    pid: PidController,
}

impl OxygenController {
    pub fn new(kp: f64, ki: f64, kd: f64, dt: f64) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd, dt),
        }
    }

    /// Regulate using fixed PID control.
    pub fn regulate(&mut self, desired: f64, measured: f64) -> f64 {
        self.pid.compute(desired, measured)
    }

    /// Regulate using adaptive PID control.
    pub fn regulate_adaptive(&mut self, desired: f64, measured: f64) -> f64 {
        self.pid.compute_adaptive(desired, measured)
    }
}

pub struct AirSupplyController {
    pid: PidController,
    /// Desired oxygen concentration setpoint (for example, 0.21 for ambient air).
    desired_oxygen: f64,
}

impl AirSupplyController {
    pub fn new(kp: f64, ki: f64, kd: f64, dt: f64, desired_oxygen: f64) -> Self {
        Self {
            pid: PidController::new(kp, ki, kd, dt),
            desired_oxygen,
        }
    }
    
    /// Compute the compressor motor torque command.
    ///
    /// A feedforward term (here, a placeholder value) is combined with a PID correction.
    pub fn compute_motor_torque(&mut self, measured_oxygen: f64) -> f64 {
        let feedforward = 10.0; // Replace with a value derived from your compressor map if available.
        let correction = self.pid.compute(self.desired_oxygen, measured_oxygen);
        feedforward + correction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_controller_output() {
        let mut pid = PidController::new(70.0, 0.3, 0.05, 0.05);
        let output = pid.compute(80.0, 70.0);
        // With an error of 10, output should be positive.
        assert!(output > 0.0);
    }
}
