#[derive(Debug)]
pub struct FuelCell {
    // Electrical state
    pub voltage: f64,       // Stack voltage [V]
    pub current: f64,       // Current drawn [A]
    pub hydrogen_flow: f64, // Hydrogen flow rate
    pub temperature: f64,   // Cell temperature [°C]

    // Base model parameters
    pub base_ocv: f64,         // Base open-circuit voltage [V]
    pub r_internal: f64,       // Base internal resistance [Ohm]
    pub thermal_mass: f64,     // Thermal mass [J/°C]
    pub cooling_efficiency: f64, // Cooling efficiency coefficient
    pub ambient_temp: f64,     // Ambient temperature [°C]

    // Detailed loss modeling parameters
    pub activation_constant: f64,    // Activation loss parameter (A) [V]
    pub exchange_current: f64,       // Exchange current (I0) [A]
    pub concentration_constant: f64, // Concentration loss parameter (B) [V]
    pub limiting_current: f64,       // Limiting current (I_lim) [A]

    // Membrane hydration state and dynamics
    pub membrane_hydration: f64,      // Hydration level (0.1 to 1.0)
    pub hydration_time_constant: f64, // Time constant for hydration dynamics [sec]

    // Temperature dependence coefficient for OCV
    pub temp_coefficient: f64, // [V/°C] drop in ocv per °C above ambient
}

impl FuelCell {
    pub fn new() -> Self {
        FuelCell {
            voltage: 60.0,
            current: 0.0,
            hydrogen_flow: 1.0,
            temperature: 45.0,
            base_ocv: 60.0,
            r_internal: 0.1,
            thermal_mass: 120.0,
            cooling_efficiency: 1.2,
            ambient_temp: 20.0,
            // Detailed loss parameters:
            activation_constant: 0.1,
            exchange_current: 0.2,
            concentration_constant: 0.08,
            limiting_current: 1.5,
            // Hydration dynamics:
            membrane_hydration: 1.0,
            hydration_time_constant: 10.0,
            // Temperature dependence:
            temp_coefficient: 0.05,
        }
    }

    /// Update the fuel cell state.
    ///
    /// Parameters:
    /// - load: current load on the stack [A]
    /// - cooling_active: whether the cooling mechanism is active
    /// - oxygen_concentration: measured oxygen concentration (0 to 1 scale)
    /// - humidity: ambient humidity or desired hydration level (0 to 1 scale)
    pub fn update(&mut self, load: f64, cooling_active: bool, oxygen_concentration: f64, humidity: f64) {
        // Update electrical current.
        self.current = load;

        // Calculate effective open-circuit voltage accounting for temperature.
        let effective_ocv = self.base_ocv - self.temp_coefficient * (self.temperature - self.ambient_temp);

        // Compute losses using a detailed polarization model:
        // Activation loss: V_act = A * ln(1 + I/I0)
        let v_act = self.activation_constant * (1.0 + load / self.exchange_current).ln();

        // Ohmic loss: effective resistance increases when hydration is low.
        let effective_r = self.r_internal / self.membrane_hydration;
        let v_ohm = load * effective_r;

        // Concentration loss: V_conc = -B * ln(1 - I/I_lim) for I < I_lim, else a high drop.
        let v_conc = if load < self.limiting_current {
            -self.concentration_constant * (1.0 - load / self.limiting_current).ln()
        } else {
            0.5
        };

        // Compute the cell voltage.
        self.voltage = effective_ocv - (v_act + v_ohm + v_conc);

        // Apply additional effects: oxygen starvation and low hydration penalty.
        if oxygen_concentration < 0.3 {
            self.voltage *= 0.85;
        }
        if self.membrane_hydration < 0.5 {
            self.voltage *= 0.9;
        }

        // Flow modeling: update hydrogen flow.
        self.hydrogen_flow = 1.0 + 0.07 * load.powf(0.9);

        // Dynamic update for membrane hydration using a first-order model.
        let dt = 0.5;
        let dh_dt = (humidity - self.membrane_hydration) / self.hydration_time_constant;
        self.membrane_hydration += dh_dt * dt;
        if self.membrane_hydration > 1.0 {
            self.membrane_hydration = 1.0;
        }
        if self.membrane_hydration < 0.1 {
            self.membrane_hydration = 0.1;
        }

        // Thermal update: account for heat generation and cooling effect.
        let heat_generated = load * 2.5;
        let effective_cooling_rate = if cooling_active {
            self.cooling_efficiency
        } else {
            0.7
        };
        self.temperature += dt * (heat_generated - effective_cooling_rate * (self.temperature - self.ambient_temp)) / self.thermal_mass;
    }

    /// Compute oxygen concentration based on hydrogen flow.
    /// This is a placeholder model and can be extended with dynamic flow effects.
    pub fn compute_oxygen_concentration(&self) -> f64 {
        0.21 * (self.hydrogen_flow / (self.hydrogen_flow + 0.5))
    }
}

#[derive(Debug)]
pub struct Battery {
    pub soc: f64,
    pub voltage: f64,
    pub current: f64,
    pub temperature: f64,
}

impl Battery {
    pub fn new() -> Self {
        Battery {
            soc: 100.0,
            voltage: 53.0,
            current: 0.0,
            temperature: 40.0,
        }
    }

    /// Update battery state given charge and discharge currents.
    pub fn update(&mut self, charge_current: f64, discharge_current: f64) {
        let net_current = charge_current - discharge_current;
        self.soc += net_current * 0.1;

        if self.soc > 100.0 {
            self.soc = 100.0;
        }
        if self.soc < 0.0 {
            self.soc = 0.0;
        }

        let r_int = 0.1;
        // Simple OCV function dependent on SoC.
        let ocv = 47.0 + 6.0 * ((self.soc / 100.0).powi(2));
        self.voltage = ocv - net_current * r_int;
        self.current = net_current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuel_cell_update_without_cooling() {
        let mut fc = FuelCell::new();
        let initial_temp = fc.temperature;
        fc.update(10.0, false, 0.5, 0.8);
        assert!(fc.temperature > initial_temp, "Temperature should rise with load");
    }

    #[test]
    fn test_fuel_cell_update_with_cooling() {
        let mut fc = FuelCell::new();
        fc.temperature = 50.0;
        fc.update(10.0, true, 0.5, 0.8);
        let temp_with_cooling = fc.temperature;
        fc.temperature = 50.0;
        fc.update(10.0, false, 0.5, 0.8);
        let temp_without_cooling = fc.temperature;
        assert!(temp_with_cooling < temp_without_cooling, "Cooling should reduce temperature rise");
    }

    #[test]
    fn test_battery_update() {
        let mut bat = Battery::new();
        let initial_soc = bat.soc;
        bat.update(2.0, 5.0);
        assert!(bat.soc < initial_soc, "Battery should discharge if discharge current is greater");
    }
}
