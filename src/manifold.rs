// manifold.rs

/// A simple manifold model using a lumped volume and the ideal gas law.
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
    /// Uses a simple mass-balance approach:
    ///   dP/dt = (R * T / V) * (mass_flow_in - mass_flow_out)
    /// where R is the gas constant for air (~287 J/(kg·K)).
    pub fn update(&mut self, mass_flow_in: f64, mass_flow_out: f64, dt: f64) {
        let r_air = 287.0;
        let dP = (r_air * self.temperature / self.volume) * (mass_flow_in - mass_flow_out) * dt;
        self.pressure += dP;
    }
}
