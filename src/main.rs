mod simulation;
mod sensors;
mod control;
mod hal;

use simulation::{Battery, FuelCell};
use sensors::{read_battery_sensor, read_fuel_cell_sensor};
use control::PidController;
use hal::{HardwareInterface, SimulatedActuator, SimulatedTemperatureSensor, DigitalOutput};

use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

fn main() {
    // Wrap the fuel cell in an Rc<RefCell> for shared mutable access.
    let fuel_cell = Rc::new(RefCell::new(FuelCell::new()));
    let mut battery = Battery::new();

    let mut pid = PidController::new(70.0, 0.3, 0.05, 0.05, 0.5);
    let charging_current = 8.0;
    let lower_threshold = 65.0;
    let upper_threshold = 75.0;
    let mut charging_mode = false;

    // We'll use a persistent flag for cooling.
    let mut cooling_active = false;

    // Create a simulated temperature sensor for the fuel cell.
    let temp_sensor = SimulatedTemperatureSensor {
        read_fn: {
            let fc = Rc::clone(&fuel_cell);
            move || fc.borrow().temperature
        },
    };

    // Create a simulated actuator (e.g., a cooling fan).
    let actuator = SimulatedActuator::new();

    // Combine into a hardware interface.
    let mut hw_interface = HardwareInterface {
        temperature_sensor: temp_sensor,
        actuator,
    };

    // Simulation loop.
    for step in 0..100 {
        // Determine charging/discharge mode based on battery SoC.
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

        // Update simulation.
        if charging_mode {
            println!("Step {}: Charging mode activated", step);
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

        // Use the hardware interface to read temperature.
        let current_temp = hw_interface.read_temperature();
        // For demonstration, we lower the threshold to 44°C.
        if current_temp > 44.0 {
            hw_interface.activate_actuator();
        } else {
            hw_interface.deactivate_actuator();
        }
        // Update our cooling flag based on actuator state.
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
