#[derive(Debug)]
pub struct FuelCell {
    pub voltage: f64,
    pub current: f64,
    pub hydrogen_flow: f64,
    pub temperature: f64,
    base_ocv: f64,
    r_internal: f64,
    thermal_mass: f64,
    cooling_efficiency: f64,
    ambient_temp: f64,
    membrane_hydration: f64,
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
            membrane_hydration: 1.0, // start fully hydrated
        }
    }

    /// Update the fuel cell state with four parameters:
    /// - load (f64): current load on the stack
    /// - cooling_active (bool): if the cooling mechanism is on
    /// - oxygen_concentration (f64): measure of available O2
    /// - humidity (f64): 0..1 range for membrane hydration
    pub fn update(&mut self, load: f64, cooling_active: bool, oxygen_concentration: f64, humidity: f64) {
        // Basic electrical model
        self.current = load;
        self.voltage = self.base_ocv - load * self.r_internal;

        // Flow modeling
        self.hydrogen_flow = 1.0 + 0.07 * load.powf(0.9);

        // Membrane hydration dynamics
        self.membrane_hydration = humidity;

        // Oxygen starvation effect
        if oxygen_concentration < 0.3 {
            self.voltage *= 0.85;
        }

        // Additional drop if membrane too dry
        if self.membrane_hydration < 0.5 {
            self.voltage *= 0.9;
        }

        // Thermal update
        let dt = 0.5;
        let heat_generated = load * 2.5;
        let effective_cooling_rate = if cooling_active {
            self.cooling_efficiency
        } else {
            0.7
        };

        self.temperature += dt
            * (heat_generated - effective_cooling_rate * (self.temperature - self.ambient_temp))
            / self.thermal_mass;
    }

    /// A simple formula for oxygen concentration (placeholder)
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

    /// Update battery state given charge current and discharge current
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
        // Simple OCV function
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
        fc.update(10.0, false);
        assert!(fc.temperature > initial_temp, "Temperature should rise with load");
    }

    #[test]
    fn test_fuel_cell_update_with_cooling() {
        let mut fc = FuelCell::new();
        fc.temperature = 50.0;
        fc.update(10.0, true);
        let temp_with_cooling = fc.temperature;
        fc.temperature = 50.0;
        fc.update(10.0, false);
        let temp_without_cooling = fc.temperature;
        assert!(temp_with_cooling < temp_without_cooling, "Cooling should reduce temperature rise");
    }

    #[test]
    fn test_battery_update() {
        let mut bat = Battery::new();
        let initial_soc = bat.soc;
        bat.update(2.0, 5.0); // Now discharge current (5.0) > charging current (2.0)
        assert!(bat.soc < initial_soc, "Battery should discharge if discharge current is greater");
    }
}
