mod simulation;
mod sensors;
mod control;
mod hal;

use simulation::{FuelCell, Battery, AirSupplySystem};
use sensors::{read_fuel_cell_sensor, read_battery_sensor};
use control::{PidController, OxygenController};
use wasm_bindgen::prelude::*; // for #[wasm_bindgen(start)]
use yew::prelude::*;          // for the Yew framework
use gloo::timers::callback::Interval; // for periodic updates/ticks

/// The main GUI model for our simulation.
struct Model {
    fuel_cell: FuelCell,
    battery: Battery,
    air_supply: AirSupplySystem,
    pid: PidController,
    oxygen_controller: OxygenController,
    charging_mode: bool,
    cooling_active: bool,
    interval: Interval,
}

/// Messages for our Yew component.
enum Msg {
    Tick,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Create the components of the simulation.
        let fuel_cell = FuelCell::new();
        let battery = Battery::new();
        let air_supply = AirSupplySystem::new();
        let pid = PidController::new(0.3, 0.05, 0.05, 0.5);
        let oxygen_controller = OxygenController::new(0.5, 0.1, 0.01, 0.5);
        let charging_mode = false;
        let cooling_active = false;

        // Set up an interval timer for periodic updates.
        let link = ctx.link().clone();
        let interval = Interval::new(500, move || {
            link.send_message(Msg::Tick);
        });

        Self {
            fuel_cell,
            battery,
            air_supply,
            pid,
            oxygen_controller,
            charging_mode,
            cooling_active,
            interval,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                // Simple SoC thresholds for switching charging mode.
                let lower_threshold = 65.0;
                let upper_threshold = 75.0;
                if self.charging_mode {
                    if self.battery.soc > upper_threshold {
                        self.charging_mode = false;
                    }
                } else {
                    if self.battery.soc < lower_threshold {
                        self.charging_mode = true;
                    }
                }

                // Read sensor data from the fuel cell and battery.
                let fc_data = read_fuel_cell_sensor(&self.fuel_cell);
                let _bat_data = read_battery_sensor(&self.battery);

                // Define disturbance and desired humidity.
                let disturbance = 10.0;
                let humidity = 0.8;

                // Determine load based on charging mode and oxygen control.
                let load = if self.charging_mode {
                    8.0 // charging current
                } else {
                    self.oxygen_controller
                        .regulate_adaptive(2.0, fc_data.oxygen_concentration)
                        + disturbance
                };

                // Activate cooling if temperature exceeds threshold.
                if self.fuel_cell.temperature > 44.0 {
                    self.cooling_active = true;
                } else {
                    self.cooling_active = false;
                }

                // Update the air supply system.
                // Assume a constant motor torque and compute mass flow out from fuel cell's hydrogen flow.
                let motor_torque = 10.0;
                let dt = 0.5;
                let mass_flow_out = self.fuel_cell.hydrogen_flow * 0.05;
                self.air_supply.update(motor_torque, dt, mass_flow_out);

                // Compute oxygen concentration based on the updated manifold pressure.
                let oxygen_concentration =
                    self.fuel_cell.compute_oxygen_concentration(self.air_supply.manifold.pressure);

                // Update fuel cell and battery states.
                self.fuel_cell
                    .update(load, self.cooling_active, oxygen_concentration, humidity);
                self.battery.update(load * 0.5, load);

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div style="font-family: sans-serif;">
                <h1>{ "BMS Simulation (Web) - Updated with Air Supply System" }</h1>
                <p>{ format!("FuelCell -> V: {:.2} V, I: {:.2} A, Temp: {:.2} °C",
                    self.fuel_cell.voltage, self.fuel_cell.current, self.fuel_cell.temperature) }</p>
                <p>{ format!("Membrane Hydration: {:.2}", self.fuel_cell.membrane_hydration) }</p>
                <p>{ format!("Manifold Pressure: {:.2} Pa", self.air_supply.manifold.pressure) }</p>
                <p>{ format!("Battery -> SoC: {:.2} %, V: {:.2} V, I: {:.2} A",
                    self.battery.soc, self.battery.voltage, self.battery.current) }</p>
                <p>{ format!("Charging Mode: {}", if self.charging_mode { "Yes" } else { "No" }) }</p>
                <p>{ format!("Cooling Active: {}", if self.cooling_active { "Yes" } else { "No" }) }</p>
            </div>
        }
    }
}

/// The WASM entry point. Called automatically after load.
#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<Model>::new().render();
}
