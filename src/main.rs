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

        /// Update the fuel cell model based on a given current.
        pub fn update(&mut self, current: f64) {
            self.current = current;
            self.voltage = 50.0 - current * 0.1;
            self.temperature += current * 0.05;
            self.hydrogen_flow = 1.0 + current * 0.01;
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
                soc: 100.0, // Starting fully charged
                voltage: 48.0,
                current: 0.0,
                temperature: 25.0,
            }
        }

        /// Update the battery state given a charge current and a discharge current.
        /// A positive net current means charging, while a negative net current means discharging.
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
}

mod control {
    /// A simple proportional controller for adjusting the load based on battery state-of-charge.
    pub struct Controller {
        pub desired_soc: f64,
        kp: f64,
    }
    
    impl Controller {
        pub fn new(desired_soc: f64, kp: f64) -> Self {
            Self { desired_soc, kp }
        }
        
        /// Computes the discharge load to be used by the simulation when the battery is above the desired SoC.
        /// 
        /// - `current_soc`: The current state-of-charge of the battery.
        /// - `disturbance`: An external load disturbance (e.g., variable load demand).
        ///
        /// If the battery is above the desired SoC, the controller produces a positive load (discharge).
        pub fn compute_load(&self, current_soc: f64, disturbance: f64) -> f64 {
            let error = current_soc - self.desired_soc; // positive when battery is overcharged
            let control_adjustment = self.kp * error;
            let load = disturbance + control_adjustment;
            if load < 0.0 { 0.0 } else { load }
        }
    }
}

use simulation::{Battery, FuelCell};
use sensors::{read_battery_sensor, read_fuel_cell_sensor};
use control::Controller;

fn main() {
    // Create simulation objects for the fuel cell and battery.
    let mut fuel_cell = FuelCell::new();
    let mut battery = Battery::new();
    
    // Create a controller with a desired battery SoC of 70% and a proportional gain of 0.5.
    let controller = Controller::new(70.0, 0.5);
    
    // Define a constant charging current value for when the battery is below the desired SoC.
    let charging_current = 5.0; // Amps

    // Run a simulation loop.
    for step in 0..100 {
        if battery.soc < controller.desired_soc {
            // Active charging mode
            println!("Step {}: Charging mode activated", step);
            // Update fuel cell to provide a constant charging current.
            fuel_cell.update(charging_current);
            // Update battery: apply charging current; no discharge.
            battery.update(charging_current, 0.0);
        } else {
            // Discharge control mode
            // Simulate an external load disturbance (varies with time).
            let disturbance = (step as f64).sin().abs() * 10.0;
            let load = controller.compute_load(battery.soc, disturbance);
            println!("Step {}: Computed load (discharge) = {:.2}", step, load);
            // Update fuel cell with the computed load.
            fuel_cell.update(load);
            // Update battery: fuel cell partially charges the battery while discharging at full load.
            battery.update(load * 0.5, load);
        }
        
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
        
        thread::sleep(Duration::from_millis(500));
    }
}
