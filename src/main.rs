use std::thread;
use std::time::Duration;

mod simulation {
    /// A simple simulation model for a hydrogen fuel cell.
    #[derive(Debug)]
    pub struct FuelCell {
        pub voltage: f64,
        pub current: f64,
        pub hydrogen_flow: f64,
        pub temperature: f64,
    }

    impl FuelCell {
        pub fn new() -> Self {
            // Initial nominal values.
            FuelCell {
                voltage: 50.0,
                current: 0.0,
                hydrogen_flow: 1.0,
                temperature: 25.0,
            }
        }

        /// Update the fuel cell model based on a given load.
        pub fn update(&mut self, load: f64) {
            self.current = load;
            self.voltage = 50.0 - load * 0.1;
            self.temperature += load * 0.05;
            self.hydrogen_flow = 1.0 + load * 0.01;
        }
    }

    /// A simple battery model that simulates state-of-charge and voltage.
    #[derive(Debug)]
    pub struct Battery {
        pub soc: f64, // State of Charge (percentage)
        pub voltage: f64,
        pub current: f64,
        pub temperature: f64,
    }

    impl Battery {
        pub fn new() -> Self {
            Battery {
                soc: 100.0,
                voltage: 48.0,
                current: 0.0,
                temperature: 25.0,
            }
        }

        /// Update the battery state given a charge and discharge current.
        pub fn update(&mut self, charge_current: f64, discharge_current: f64) {
            // Calculate net current (charging increases SoC, discharging decreases it)
            let net_current = charge_current - discharge_current;
            self.soc += net_current * 0.1; // Simple proportional update

            // Clamp SoC between 0% and 100%
            if self.soc > 100.0 {
                self.soc = 100.0;
            } else if self.soc < 0.0 {
                self.soc = 0.0;
            }

            self.current = net_current;
            // Update voltage as a function of state-of-charge (simplified)
            self.voltage = 48.0 + (self.soc - 50.0) * 0.1;
            self.temperature += discharge_current * 0.05;
        }
    }
}

mod sensors {
    use crate::simulation::{Battery, FuelCell};

    /// Structure representing the sensor data for the fuel cell.
    #[derive(Debug)]
    pub struct FuelCellSensorData {
        pub voltage: f64,
        pub current: f64,
        pub hydrogen_flow: f64,
        pub temperature: f64,
    }

    /// Structure representing the sensor data for the battery.
    #[derive(Debug)]
    pub struct BatterySensorData {
        pub soc: f64,
        pub voltage: f64,
        pub current: f64,
        pub temperature: f64,
    }

    /// Emulate reading sensor data from the fuel cell.
    pub fn read_fuel_cell_sensor(fuel_cell: &FuelCell) -> FuelCellSensorData {
        FuelCellSensorData {
            voltage: fuel_cell.voltage,
            current: fuel_cell.current,
            hydrogen_flow: fuel_cell.hydrogen_flow,
            temperature: fuel_cell.temperature,
        }
    }

    /// Emulate reading sensor data from the battery.
    pub fn read_battery_sensor(battery: &Battery) -> BatterySensorData {
        BatterySensorData {
            soc: battery.soc,
            voltage: battery.voltage,
            current: battery.current,
            temperature: battery.temperature,
        }
    }
}

mod hal {
    // In this phase, we create a stub for our hardware abstraction layer.
    // Later, you can implement actual embedded-hal traits to interface with hardware sensors.
    pub trait Sensor {
        type Output;
        fn read(&self) -> Self::Output;
    }

    // For now, our sensor readings are provided by our simulation.
    // This module lays the groundwork for integrating real sensor interfaces later.
}

use simulation::{Battery, FuelCell};
use sensors::{read_battery_sensor, read_fuel_cell_sensor};

fn main() {
    // Create simulation objects for the fuel cell and battery.
    let mut fuel_cell = FuelCell::new();
    let mut battery = Battery::new();

    // Run a simulation loop.
    for step in 0..100 {
        // Generate a varying load (using a sine function for variability).
        let load = (step as f64).sin().abs() * 20.0;

        // Update the simulation models.
        fuel_cell.update(load);
        // Assume the fuel cell partially charges the battery while the battery is discharging under load.
        battery.update(load * 0.5, load);

        // Emulate sensor readings.
        let fc_data = read_fuel_cell_sensor(&fuel_cell);
        let bat_data = read_battery_sensor(&battery);

        println!("Step {}:", step);
        println!(
            "  Fuel Cell -> Voltage: {:.2} V, Current: {:.2} A, Hydrogen Flow: {:.2}, Temp: {:.2} °C",
            fc_data.voltage, fc_data.current, fc_data.hydrogen_flow, fc_data.temperature
        );
        println!(
            "  Battery   -> SoC: {:.2} %, Voltage: {:.2} V, Current: {:.2} A, Temp: {:.2} °C",
            bat_data.soc, bat_data.voltage, bat_data.current, bat_data.temperature
        );

        // Pause to simulate time steps.
        thread::sleep(Duration::from_millis(500));
    }
}
