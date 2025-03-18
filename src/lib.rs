mod simulation;
mod sensors;
mod control;
mod hal;
//mod controllers; // Your controllers folder containing AirSupplyController and BatteryController

use simulation::{FuelCell, Battery, AirSupplySystem};
use sensors::{read_fuel_cell_sensor, read_battery_sensor};
use control::{OxygenController, AirSupplyController, BatteryController}; // Removed unused PidController import
use wasm_bindgen::prelude::*; // for #[wasm_bindgen(start)]
use yew::prelude::*;          // for Yew components
use gloo::timers::callback::Interval; // for periodic updates

/// The main GUI model for our simulation.
struct Model {
    fuel_cell: FuelCell,
    battery: Battery,
    air_supply: AirSupplySystem,
    oxygen_controller: OxygenController,
    air_supply_controller: AirSupplyController, // our new controller
    battery_controller: BatteryController,       // battery SoC controller
    charging_mode: bool,
    cooling_active: bool,
    interval: Interval,
    debug_log: Vec<String>, // Accumulated debug output
}

/// Messages for our Yew component.
enum Msg {
    Tick,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Create simulation components.
        let fuel_cell = FuelCell::new();
        let battery = Battery::new();
        let air_supply = AirSupplySystem::new();
        let oxygen_controller = OxygenController::new(0.5, 0.1, 0.01, 0.5);
        let air_supply_controller = AirSupplyController::new(0.5, 0.05, 0.05, 0.5, 0.21);
        let battery_controller = BatteryController::new(65.0, 75.0);
        let charging_mode = false;
        let cooling_active = false;
        let debug_log = Vec::new();
        let link = ctx.link().clone();
        let interval = Interval::new(500, move || {
            link.send_message(Msg::Tick);
        });
        Self {
            fuel_cell,
            battery,
            air_supply,
            oxygen_controller,
            air_supply_controller,
            battery_controller,
            charging_mode,
            cooling_active,
            interval,
            debug_log,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                // Update battery mode (hysteresis-based).
                self.charging_mode = self.battery_controller.update_mode(self.battery.soc);

                // Read sensor data.
                let fc_data = read_fuel_cell_sensor(&self.fuel_cell);

                // Compute compressor motor torque from AirSupplyController.
                let motor_torque = self.air_supply_controller.compute_motor_torque(fc_data.oxygen_concentration);

                // Define time step and estimate mass flow out.
                let dt = 0.5;
                let mass_flow_out = self.fuel_cell.hydrogen_flow * 0.05;
                let is_discharging = !self.charging_mode;
                self.air_supply.update(motor_torque, dt, mass_flow_out, is_discharging);

                // Compute oxygen concentration from updated manifold pressure.
                let oxygen_concentration = self.fuel_cell.compute_oxygen_concentration_from(self.air_supply.manifold.pressure);

                // Determine load using oxygen controller and disturbance.
                let disturbance = 10.0;
                let load = if self.charging_mode {
                    8.0 // fixed charging current
                } else {
                    self.oxygen_controller.regulate_adaptive(2.0, fc_data.oxygen_concentration) + disturbance
                };

                // Set cooling based on temperature.
                self.cooling_active = self.fuel_cell.temperature > 44.0;

                // Update fuel cell state.
                let humidity = 0.8; // Base humidity value
                self.fuel_cell.update(load, self.cooling_active, oxygen_concentration, humidity);

                // Update battery state based on mode.
                if self.charging_mode {
                    self.battery.update(8.0, 0.0, true);
                } else {
                    self.battery.update(0.0, load, false);
                }

                // Append a new debug log entry.
                let log_entry = format!(
                    "V: {:.2} V, I: {:.2} A, Temp: {:.2} °C, Hydration: {:.2}, SOC: {:.2}%, MPress: {:.2} Pa, O2: {:.2}",
                    self.fuel_cell.voltage,
                    self.fuel_cell.current,
                    self.fuel_cell.temperature,
                    self.fuel_cell.membrane_hydration,
                    self.battery.soc,
                    self.air_supply.manifold.pressure,
                    oxygen_concentration,
                );
                self.debug_log.push(log_entry);
                if self.debug_log.len() > 50 {
                    self.debug_log.drain(0..(self.debug_log.len() - 50));
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let debug_text = self.debug_log.join("\n");
        html! {
            <div style="font-family: sans-serif;">
                <h1>{ "BMS Simulation (Web) - Debug Output" }</h1>
                <p>{ format!("FuelCell -> V: {:.2} V, I: {:.2} A, Temp: {:.2} °C",
                    self.fuel_cell.voltage, self.fuel_cell.current, self.fuel_cell.temperature) }</p>
                <p>{ format!("Membrane Hydration: {:.2}", self.fuel_cell.membrane_hydration) }</p>
                <p>{ format!("Manifold Pressure: {:.2} Pa", self.air_supply.manifold.pressure) }</p>
                <p>{ format!("Oxygen Concentration: {:.2}", self.fuel_cell.oxygen_concentration) }</p>
                <p>{ format!("Battery -> SoC: {:.2} %, V: {:.2} V, I: {:.2} A",
                    self.battery.soc, self.battery.voltage, self.battery.current) }</p>
                <p>{ format!("Charging Mode: {}", if self.charging_mode { "Yes" } else { "No" }) }</p>
                <p>{ format!("Cooling Active: {}", if self.cooling_active { "Yes" } else { "No" }) }</p>
                <h2>{ "Debug Log:" }</h2>
                <pre style="background-color: #f0f0f0; padding: 10px; max-height: 300px; overflow-y: scroll;">
                    { debug_text }
                </pre>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<Model>::new().render();
}
