mod simulation;
mod sensors;
mod control;
mod hal;
//mod controllers; // New controllers module

use simulation::{FuelCell, Battery, AirSupplySystem};
use sensors::{read_fuel_cell_sensor, read_battery_sensor};
use control::{PidController, OxygenController, AirSupplyController, BatteryController};
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
    battery_controller: BatteryController,      // our new battery controller
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
        // Create AirSupplyController with PID gains and desired oxygen concentration.
        let air_supply_controller = AirSupplyController::new(0.5, 0.05, 0.05, 0.5, 0.21);
        // Create BatteryController using the provided constructor.
        let battery_controller = BatteryController::new(65.0, 75.0);
        let debug_log = Vec::new(); // Start with an empty log
        let charging_mode = false;
        let cooling_active = false;
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
            charging_mode,
            cooling_active,
            interval,
            battery_controller,
            debug_log,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                // Update charging mode using BatteryController (hysteresis).
                self.charging_mode = self.battery_controller.update_mode(self.battery.soc);
                
                // Read fuel cell sensor data.
                let fc_data = read_fuel_cell_sensor(&self.fuel_cell);
                
                // Use the AirSupplyController to compute compressor motor torque.
                let motor_torque = self.air_supply_controller.compute_motor_torque(fc_data.oxygen_concentration);
                
                // Update the air supply system (compressor and manifold).
                let dt = 0.5;
                let mass_flow_out = self.fuel_cell.hydrogen_flow * 0.05; // Estimate of air consumption.
                self.air_supply.update(motor_torque, dt, mass_flow_out);
                
                // Compute oxygen concentration from updated manifold pressure.
                let oxygen_concentration = self.fuel_cell.compute_oxygen_concentration_from(self.air_supply.manifold.pressure);
                
                // Use disturbance and oxygen controller to determine load.
                let disturbance = 10.0;
                let load = if self.charging_mode {
                    8.0 // Use a fixed load for charging mode.
                } else {
                    self.oxygen_controller.regulate_adaptive(2.0, fc_data.oxygen_concentration) + disturbance
                };
                
                // Determine cooling based on fuel cell temperature.
                self.cooling_active = self.fuel_cell.temperature > 44.0;
                
                // Update fuel cell state using the new oxygen concentration.
                let humidity = 0.8; // Base humidity.
                self.fuel_cell.update(load, self.cooling_active, oxygen_concentration, humidity);
                
                // Update battery state:
                // If charging_mode is true, apply a positive charge current; otherwise, discharge.
                if self.charging_mode {
                    self.battery.update(8.0, 0.0);
                } else {
                    self.battery.update(0.0, load);
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
                
                // Limit log size to the latest 50 entries.
                if self.debug_log.len() > 50 {
                    self.debug_log.drain(0..(self.debug_log.len() - 50));
                }
                
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // Join the debug log entries into one string separated by newlines.
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
