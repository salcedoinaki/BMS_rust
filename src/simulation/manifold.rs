#[derive(Debug)]
pub struct Manifold {
    /// Current pressure in the manifold [Pa]
    pub pressure: f64,
    /// Manifold volume [m³]
    pub volume: f64,
    /// Manifold temperature [K]
    pub temperature: f64,
}

impl Manifold {
    /// Create a new manifold with given volume, temperature, and initial pressure.
    pub fn new(volume: f64, temperature: f64, initial_pressure: f64) -> Self {
        Self {
            volume,
            temperature,
            pressure: initial_pressure,
        }
    }
    
    /// Update the manifold pressure.
    ///
    /// The update includes:
    /// - A mass-balance term.
    /// - A baseline leak term.
    /// - An active vent term that kicks in above a threshold.
    /// - An additional proportional control term that further reduces pressure
    ///   when it exceeds the target.
    ///
    /// The parameters are tuned to be stronger in discharge mode.
    ///
    /// dt: time step [s]
    /// is_discharging: true when the system is in discharge mode.
    pub fn update(&mut self, mass_flow_in: f64, mass_flow_out: f64, dt: f64, is_discharging: bool) {
        let r_air = 287.0;
        let ambient_pressure = 101325.0;
        let target_pressure = 380000.0; // 4 bar target
        
        // Mass balance: increase pressure if inflow exceeds outflow.
        let dP_mass = (r_air * self.temperature / self.volume) * (mass_flow_in - mass_flow_out) * dt;
        
        // Baseline leak: continuously vent a fraction of the excess pressure.
        let k_leak = 0.05;
        let dP_leak = k_leak * (self.pressure - ambient_pressure) * dt;
        
        // Active vent: if pressure exceeds the target, vent extra pressure.
        let k_vent = if is_discharging { 0.1 } else { 0.05 };
        let dP_vent = if self.pressure > target_pressure {
            k_vent * (self.pressure - target_pressure) * dt
        } else {
            0.0
        };
        
        // Additional control term: proportional feedback that subtracts pressure
        // proportional to the error above target. Use a higher gain during discharge.
        let k_control = if is_discharging { 0.2 } else { 0.1 };
        let dP_control = if self.pressure > target_pressure {
            k_control * (self.pressure - target_pressure) * dt
        } else {
            0.0
        };
        
        self.pressure += dP_mass - dP_leak - dP_vent - dP_control;
        
        // Ensure pressure does not fall below ambient.
        if self.pressure < ambient_pressure {
            self.pressure = ambient_pressure;
        }
    }
}
