use crate::simulation::{Battery, FuelCell};

#[derive(Debug)]
pub struct FuelCellSensorData {
    pub voltage: f64,
    pub current: f64,
    pub hydrogen_flow: f64,
    pub temperature: f64,
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
