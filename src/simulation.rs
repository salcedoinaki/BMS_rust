#[derive(Debug)]
pub struct FuelCell {
    pub voltage: f64,
    pub current: f64,
    pub hydrogen_flow: f64,
    pub temperature: f64,
    base_ocv: f64,
    r_internal: f64,
}

impl FuelCell {
    pub fn new() -> Self {
        FuelCell {
            voltage: 52.0,
            current: 0.0,
            hydrogen_flow: 1.0,
            temperature: 40.0, // starting at 40°C
            base_ocv: 52.0,
            r_internal: 0.2,
        }
    }

    // Now accepts a cooling_active flag.
    pub fn update(&mut self, load: f64, cooling_active: bool) {
        self.current = load;
        self.voltage = self.base_ocv - load * self.r_internal;
        self.hydrogen_flow = 1.0 + 0.05 * load;
        let dt = 0.5;
        let ambient = 25.0;
        let thermal_mass = 100.0;
        let heat_generated = load * 2.0;
        let effective_cooling_rate = if cooling_active { 1.0 } else { 0.5 };
        self.temperature = self.temperature
            + dt * (heat_generated - effective_cooling_rate * (self.temperature - ambient)) / thermal_mass;
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
            temperature: 40.0, // starting at 40°C
        }
    }

    pub fn ocv(&self) -> f64 {
        47.0 + 6.0 * ((self.soc / 100.0).powi(2))
    }

    pub fn update(&mut self, charge_current: f64, discharge_current: f64) {
        let net_current = charge_current - discharge_current;
        self.soc += net_current * 0.1;
        if self.soc > 100.0 { self.soc = 100.0; }
        if self.soc < 0.0 { self.soc = 0.0; }
        let r_int = 0.1;
        self.voltage = self.ocv() - net_current * r_int;
        self.current = net_current;

        let dt = 0.5;
        let ambient = 25.0;
        let thermal_mass = 80.0;
        let heat_generated = discharge_current * 4.0;
        let cooling_rate = 0.25;
        self.temperature = self.temperature
            + dt * (heat_generated - cooling_rate * (self.temperature - ambient)) / thermal_mass;
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
        bat.update(5.0, 2.0);
        assert!(bat.soc < initial_soc, "Battery should discharge if discharge current is greater");
    }
}
