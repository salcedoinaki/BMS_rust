mod simulation;
mod sensors;
mod control;
mod hal;

use simulation::{Battery, FuelCell};
use sensors::{read_battery_sensor, read_fuel_cell_sensor};
use control::OxygenController;
use hal::{HardwareInterface, SimulatedActuator, SimulatedTemperatureSensor};

use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

fn main() {
    let fuel_cell = Rc::new(RefCell::new(FuelCell::new()));
    let mut battery = Battery::new();
    let mut oxygen_controller = OxygenController::new(0.5, 0.1, 0.01, 0.5);
    let mut cooling_active = false;
    let charging_current = 8.0;
    let lower_threshold = 65.0;
    let upper_threshold = 75.0;
    let mut charging_mode = false;

    let temp_sensor = SimulatedTemperatureSensor {
        read_fn: {
            let fc = Rc::clone(&fuel_cell);
            move || fc.borrow().temperature
        },
    };

    let actuator = SimulatedActuator::new();
    let mut hw_interface = HardwareInterface {
        temperature_sensor: temp_sensor,
        actuator,
    };

    for step in 0..100 {
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

        let fuel_data = read_fuel_cell_sensor(&fuel_cell.borrow());
        let humidity = 0.8;
        let disturbance = (step as f64).sin().abs() * 10.0;
        let load = if charging_mode {
            charging_current
        } else {
            // Use adaptive oxygen control
            oxygen_controller
                .regulate_adaptive(2.0, fuel_data.oxygen_concentration)
                + disturbance
        };

        fuel_cell
            .borrow_mut()
            .update(load, cooling_active, fuel_data.oxygen_concentration, humidity);
        battery.update(load * 0.5, load);

        if hw_interface.read_temperature() > 44.0 {
            hw_interface.activate_actuator();
        } else {
            hw_interface.deactivate_actuator();
        }
        cooling_active = hw_interface.get_actuator_state();

        println!("Step {}:", step);
        println!(
            "  Fuel Cell -> Voltage: {:.2} V, Current: {:.2} A, Hydrogen Flow: {:.2}, Temp: {:.2} °C",
            fuel_data.voltage, fuel_data.current, fuel_data.hydrogen_flow, fuel_data.temperature
        );
        println!(
            "  Battery   -> SoC: {:.2} %, Voltage: {:.2} V, Current: {:.2} A, Temp: {:.2} °C",
            battery.soc, battery.voltage, battery.current, battery.temperature
        );

        thread::sleep(Duration::from_millis(500));
    }
}
