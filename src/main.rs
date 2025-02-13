use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

mod simulation {
    /// A fuel cell model with a base open-circuit voltage, internal resistance,
    /// and improved temperature dynamics that now depend on a cooling flag.
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

        /// Update the fuel cell based on the load and cooling status.
        /// When `cooling_active` is true, the cooling effect is stronger.
        pub fn update(&mut self, load: f64, cooling_active: bool) {
            self.current = load;
            self.voltage = self.base_ocv - load * self.r_internal;
            self.hydrogen_flow = 1.0 + 0.05 * load;
            let dt = 0.5;           // time step in seconds
            let ambient = 25.0;     // ambient temperature in °C
            let thermal_mass = 100.0;
            let heat_generated = load * 2.0;
            // Use a higher cooling rate if the actuator (fan) is active.
            let effective_cooling_rate = if cooling_active { 1.0 } else { 0.5 };
            self.temperature = self.temperature
                + dt * (heat_generated - effective_cooling_rate * (self.temperature - ambient)) / thermal_mass;
        }
    }

    /// A battery model with a non-linear OCV, internal resistance, and temperature dynamics.
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
}

mod sensors {
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
}

mod control {
    /// A PID controller that adjusts the load based on battery SoC.
    pub struct PidController {
        pub desired_soc: f64,
        kp: f64,
        ki: f64,
        kd: f64,
        last_error: f64,
        integral: f64,
        dt: f64,
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

mod hal {
    // Define a generic sensor trait.
    pub trait Sensor {
        type Output;
        fn read(&self) -> Self::Output;
    }

    // Define a trait for digital outputs (actuators).
    pub trait DigitalOutput {
        fn set_high(&mut self);
        fn set_low(&mut self);
        fn get_state(&self) -> bool;
    }

    // A simulated temperature sensor using a closure.
    pub struct SimulatedTemperatureSensor<F>
    where
        F: Fn() -> f64,
    {
        pub read_fn: F,
    }

    impl<F> Sensor for SimulatedTemperatureSensor<F>
    where
        F: Fn() -> f64,
    {
        type Output = f64;
        fn read(&self) -> Self::Output {
            (self.read_fn)()
        }
    }

    // A simulated digital actuator (e.g., a cooling fan).
    pub struct SimulatedActuator {
        pub state: bool,
    }

    impl SimulatedActuator {
        pub fn new() -> Self {
            Self { state: false }
        }
    }

    impl DigitalOutput for SimulatedActuator {
        fn set_high(&mut self) {
            self.state = true;
            println!("Actuator set to HIGH");
        }
        fn set_low(&mut self) {
            self.state = false;
            println!("Actuator set to LOW");
        }
        fn get_state(&self) -> bool {
            self.state
        }
    }

    // A higher-level hardware interface combining a sensor and an actuator.
    pub struct HardwareInterface<T, U>
    where
        T: Sensor<Output = f64>,
        U: DigitalOutput,
    {
        pub temperature_sensor: T,
        pub actuator: U,
    }

    impl<T, U> HardwareInterface<T, U>
    where
        T: Sensor<Output = f64>,
        U: DigitalOutput,
    {
        pub fn read_temperature(&self) -> f64 {
            self.temperature_sensor.read()
        }
        pub fn activate_actuator(&mut self) {
            self.actuator.set_high();
        }
        pub fn deactivate_actuator(&mut self) {
            self.actuator.set_low();
        }
        pub fn get_actuator_state(&self) -> bool {
            self.actuator.get_state()
        }
    }
}

use simulation::{Battery, FuelCell};
use sensors::{read_battery_sensor, read_fuel_cell_sensor};
use control::PidController;
use hal::{DigitalOutput, HardwareInterface, SimulatedActuator, SimulatedTemperatureSensor};

fn main() {
    // Wrap the fuel cell in an Rc<RefCell> for shared mutable access.
    let fuel_cell = Rc::new(RefCell::new(FuelCell::new()));
    let mut battery = Battery::new();

    let mut pid = PidController::new(70.0, 0.3, 0.05, 0.05, 0.5);
    let charging_current = 8.0;
    let lower_threshold = 65.0;
    let upper_threshold = 75.0;
    let mut charging_mode = false;

    // We'll use a persistent flag for cooling. Initially, it's false.
    let mut cooling_active = false;

    // Create a simulated temperature sensor for the fuel cell.
    let temp_sensor = SimulatedTemperatureSensor {
        read_fn: {
            let fc = Rc::clone(&fuel_cell);
            move || fc.borrow().temperature
        },
    };

    // Create a simulated actuator.
    let actuator = SimulatedActuator::new();

    let mut hw_interface = HardwareInterface {
        temperature_sensor: temp_sensor,
        actuator,
    };

    // Simulation loop.
    for step in 0..100 {
        // Decide on the charging/discharge mode based on battery SoC.
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

        // Update the fuel cell and battery.
        if charging_mode {
            println!("Step {}: Charging mode activated", step);
            // Pass the cooling flag into the update.
            fuel_cell.borrow_mut().update(charging_current, cooling_active);
            battery.update(charging_current, 0.0);
        } else {
            let disturbance = (step as f64).sin().abs() * 10.0;
            let load = pid.compute_load(battery.soc, disturbance);
            println!("Step {}: Computed discharge load = {:.2}", step, load);
            fuel_cell.borrow_mut().update(load, cooling_active);
            battery.update(load * 0.5, load);
        }

        // Read sensor values.
        let fc_data = read_fuel_cell_sensor(&fuel_cell.borrow());
        let bat_data = read_battery_sensor(&battery);

        // Use our hardware interface to read temperature.
        let current_temp = hw_interface.read_temperature();
        // For demonstration, lower the threshold to 44°C to activate the actuator.
        if current_temp > 44.0 {
            hw_interface.activate_actuator();
        } else {
            hw_interface.deactivate_actuator();
        }
        // Update our cooling flag with the actuator state for next iteration.
        cooling_active = hw_interface.get_actuator_state();

        println!("Step {}:", step);
        println!(
            "  Fuel Cell -> Voltage: {:.2} V, Current: {:.2} A, Hydrogen Flow: {:.2}, Temp: {:.2} °C",
            fc_data.voltage, fc_data.current, fc_data.hydrogen_flow, fc_data.temperature
        );
        println!(
            "  Battery   -> SoC: {:.2} %, Voltage: {:.2} V, Current: {:.2} A, Temp: {:.2} °C",
            bat_data.soc, bat_data.voltage, bat_data.current, bat_data.temperature
        );
        println!(
            "  [HAL] Temperature Sensor: {:.2} °C, Actuator: {}",
            current_temp,
            if hw_interface.get_actuator_state() { "Activated" } else { "Deactivated" }
        );

        thread::sleep(Duration::from_millis(500));
    }
}
