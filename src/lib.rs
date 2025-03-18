mod simulation;
mod sensors;
mod control;
mod hal;

use simulation::{FuelCell, Battery};
use sensors::{read_fuel_cell_sensor, read_battery_sensor};
use control::{PidController, OxygenController};
use wasm_bindgen::prelude::*; // for #[wasm_bindgen(start)]
use yew::prelude::*;          // for the Yew framework
use gloo::timers::callback::Interval; // for periodic updates/ticks

/// A Yew component (the main GUI model).
struct Model {
    fuel_cell: FuelCell,
    battery: Battery,
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
        // Construct your data here
        let fuel_cell = FuelCell::new();
        let battery = Battery::new();
        let pid = PidController::new(0.3, 0.05, 0.05, 0.5);
        let oxygen_controller = OxygenController::new(0.5, 0.1, 0.01, 0.5);

        let charging_mode = false;
        let cooling_active = false;

        // Create an interval to send ticks
        let link = ctx.link().clone();
        let interval = Interval::new(500, move || {
            link.send_message(Msg::Tick);
        });

        Self {
            fuel_cell,
            battery,
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
                // Example update logic for the simulation
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

                // Read sensors
                let fc_data = read_fuel_cell_sensor(&self.fuel_cell);
                let _bat_data = read_battery_sensor(&self.battery);

                // Disturbance and humidity
                let disturbance = 10.0;
                let humidity = 0.8;

                // Decide load
                let load = if self.charging_mode {
                    8.0 // charging current
                } else {
                    // Using adaptive control for oxygen regulation
                    self.oxygen_controller
                        .regulate_adaptive(2.0, fc_data.oxygen_concentration)
                        + disturbance
                };

                // Cooling control based on temperature
                if self.fuel_cell.temperature > 44.0 {
                    self.cooling_active = true;
                } else {
                    self.cooling_active = false;
                }

                // Update the fuel cell and battery states
                self.fuel_cell.update(
                    load,
                    self.cooling_active,
                    fc_data.oxygen_concentration,
                    humidity,
                );
                self.battery.update(load * 0.5, load);

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div style="font-family: sans-serif;">
                <h1>{ "BMS Simulation (Web)" }</h1>
                <p>{ format!(
                    "FuelCell -> V: {:.2} V, I: {:.2} A, Temp: {:.2} °C",
                    self.fuel_cell.voltage,
                    self.fuel_cell.current,
                    self.fuel_cell.temperature
                )}</p>
                <p>{ format!(
                    "Battery  -> SoC: {:.2} %, V: {:.2} V, I: {:.2} A",
                    self.battery.soc,
                    self.battery.voltage,
                    self.battery.current
                )}</p>
                <p>{ format!(
                    "Charging Mode: {}",
                    if self.charging_mode { "Yes" } else { "No" }
                )}</p>
                <p>{ format!(
                    "Cooling Active: {}",
                    if self.cooling_active { "Yes" } else { "No" }
                )}</p>
            </div>
        }
    }
}

/// The WASM entry point. Called automatically after load.
#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<Model>::new().render();
}
