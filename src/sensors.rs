use crate::simulation::{FuelCell, Battery};

#[derive(Debug)]
pub struct FuelCellSensorData {
    pub voltage: f64,
    pub current: f64,
    pub hydrogen_flow: f64,
    pub temperature: f64,
    pub oxygen_concentration: f64,
}

#[derive(Debug)]
pub struct BatterySensorData {
    pub soc: f64,
    pub voltage: f64,
    pub current: f64,
    pub temperature: f64,
}

pub fn read_fuel_cell_sensor(fuel_cell: &FuelCell) -> FuelCellSensorData {
    FuelCellSensorData {
        voltage: fuel_cell.voltage,
        current: fuel_cell.current,
        hydrogen_flow: fuel_cell.hydrogen_flow,
        temperature: fuel_cell.temperature,
        oxygen_concentration: fuel_cell.oxygen_concentration,
    }
}

pub fn read_battery_sensor(battery: &Battery) -> BatterySensorData {
    BatterySensorData {
        soc: battery.soc,
        voltage: battery.voltage,
        current: battery.current,
        temperature: battery.temperature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{Battery, FuelCell};

    #[test]
    fn test_read_fuel_cell_sensor() {
        let fc = FuelCell::new();
        let data = read_fuel_cell_sensor(&fc);
        assert_eq!(data.voltage, 60.0);
    }

    #[test]
    fn test_read_battery_sensor() {
        let bat = Battery::new();
        let data = read_battery_sensor(&bat);
        assert_eq!(data.soc, 100.0);
    }
}
