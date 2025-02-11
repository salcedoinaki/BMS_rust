use std::thread;
use std::time::Duration;

mod simulation {
    /// A fuel cell model with a base open-circuit voltage, internal resistance, and improved temperature dynamics.
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
                // Assume the fuel cell stack has an open-circuit voltage of about 52 V.
                voltage: 52.0,
                current: 0.0,
                hydrogen_flow: 1.0,
                temperature: 40.0, // starting at 40°C for more dynamic behavior
                base_ocv: 52.0,
                r_internal: 0.2, // internal resistance in ohms
            }
        }

        /// Update the fuel cell based on the load (current draw).
        /// The terminal voltage is given by: V = base_ocv - I * r_internal.
        /// Hydrogen flow is updated linearly.
        /// Temperature is updated with a simple thermal model.
        pub fn update(&mut self, load: f64) {
            self.current = load;
            self.voltage = self.base_ocv - load * self.r_internal;
            self.hydrogen_flow = 1.0 + 0.05 * load;
            
            // Improved temperature model for the fuel cell:
            let dt = 0.5;            // time step (seconds)
            let ambient = 25.0;      // ambient temperature (°C)
            let thermal_mass = 100.0;  // thermal mass (higher means slower temperature change)
            // Adjusted heat generation and cooling:
            let heat_generated = load * 2.0;  // reduced heat generation multiplier
            let cooling_rate = 0.5;           // increased cooling rate
            self.temperature = self.temperature 
                + dt * (heat_generated - cooling_rate * (self.temperature - ambient)) / thermal_mass;
        }
    }

    /// A battery model with a non-linear open-circuit voltage (OCV), internal resistance, and improved temperature dynamics.
    #[derive(Debug)]
    pub struct Battery {
        pub soc: f64, // State of Charge (percentage 0 to 100)
        pub voltage: f64,
        pub current: f64,
        pub temperature: f64,
    }

    impl Battery {
        pub fn new() -> Self {
            Battery {
                soc: 100.0,  // Starting fully charged
                voltage: 53.0,
                current: 0.0,
                temperature: 40.0, // starting at 40°C
            }
        }

        /// Compute the open-circuit voltage (OCV) as a non-linear function of SoC.
        /// For example: V_oc = 47 V + 6 V * (SoC/100)^2.
        pub fn ocv(&self) -> f64 {
            47.0 + 6.0 * ((self.soc / 100.0).powi(2))
        }

        /// Update the battery state based on charging and discharging currents.
        /// Net current (charge_current - discharge_current) increases SoC if positive,
        /// and decreases SoC if negative.
        /// Terminal voltage is given by the OCV minus a drop due to internal resistance.
        /// Temperature is updated using a simple thermal model.
        pub fn update(&mut self, charge_current: f64, discharge_current: f64) {
            let net_current = charge_current - discharge_current;
            self.soc += net_current * 0.1; // update factor (depends on capacity/time step)
            if self.soc > 100.0 {
                self.soc = 100.0;
            } else if self.soc < 0.0 {
                self.soc = 0.0;
            }
            // Assume an internal resistance of 0.1 ohm.
            let r_int = 0.1;
            self.voltage = self.ocv() - net_current * r_int;
            self.current = net_current;
            
            // Improved temperature model for the battery:
            let dt = 0.5;           // time step in seconds
            let ambient = 25.0;     // ambient temperature in °C
            let thermal_mass = 80.0; // thermal mass for the battery
            let heat_generated = discharge_current * 4.0; // heat generated mainly during discharge
            let cooling_rate = 0.25; // cooling rate multiplier for the battery
            self.temperature = self.temperature 
                + dt * (heat_generated - cooling_rate * (self.temperature - ambient)) / thermal_mass;
        }
    }
}

mod sensors {
    use crate::simulation::{Battery, FuelCell};

    /// Sensor data from the fuel cell.
    #[derive(Debug)]
    pub struct FuelCellSensorData {
        pub voltage: f64,
        pub current: f64,
        pub hydrogen_flow: f64,
        pub temperature: f64,
    }

    /// Sensor data from the battery.
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
    // Stub for our hardware abstraction layer.
    pub trait Sensor {
        type Output;
        fn read(&self) -> Self::Output;
    }
}

mod control {
    /// A PID controller that adjusts the load based on battery SoC.
    /// Control action: control = kp * error + ki * integral + kd * derivative,
    /// where error = current_soc - desired_soc.
    pub struct PidController {
        pub desired_soc: f64,
        kp: f64,
        ki: f64,
        kd: f64,
        last_error: f64,
        integral: f64,
        dt: f64, // time step in seconds
    }
    
    impl PidController {
        pub fn new(desired_soc: f64, kp: f64, ki: f64, kd: f64, dt: f64) -> Self {
            Self {
                desired_soc,
                kp,
                ki,
                kd,
                last_error: 0.0,
                integral: 0.0,
                dt,
            }
        }
        
        /// Compute the discharge load using PID control when the battery is above the desired SoC.
        pub fn compute_load(&mut self, current_soc: f64, disturbance: f64) -> f64 {
            let error = current_soc - self.desired_soc;
            self.integral += error * self.dt;
            let derivative = (error - self.last_error) / self.dt;
            self.last_error = error;
            let control_adjustment = self.kp * error + self.ki * self.integral + self.kd * derivative;
            let load = disturbance + control_adjustment;
            if load < 0.0 { 0.0 } else { load }
        }
    }
}

use simulation::{Battery, FuelCell};
use sensors::{read_battery_sensor, read_fuel_cell_sensor};
use control::PidController;

fn main() {
    // Create simulation objects.
    let mut fuel_cell = FuelCell::new();
    let mut battery = Battery::new();
    
    // Create a PID controller with desired SoC 70% and tuned gains (dt = 0.5 sec).
    let mut pid = PidController::new(70.0, 0.3, 0.05, 0.05, 0.5);
    
    // Use an increased charging current for faster recovery.
    let charging_current = 8.0; // Amps
    
    // Hysteresis thresholds.
    let lower_threshold = 65.0;
    let upper_threshold = 75.0;
    let mut charging_mode = false;
    
    // Simulation loop.
    for step in 0..100 {
        // Update hysteresis mode based on current battery SoC.
        if charging_mode {
            if battery.soc > upper_threshold {
                charging_mode = false;
                println!("Step {}: Switching to discharge mode", step);
            }
        } else {
            if battery.soc < lower_threshold {
                charging_mode = true;
                println!("Step {}: Switching to charging mode", step);
            }
        }
        
        if charging_mode {
            println!("Step {}: Charging mode activated", step);
            fuel_cell.update(charging_current);
            battery.update(charging_current, 0.0);
        } else {
            let disturbance = (step as f64).sin().abs() * 10.0;
            let load = pid.compute_load(battery.soc, disturbance);
            println!("Step {}: Computed discharge load = {:.2}", step, load);
            fuel_cell.update(load);
            battery.update(load * 0.5, load);
        }
        
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
