mod simulation;
mod sensors;
mod control;
mod hal;

use simulation::{FuelCell, Battery};             // if you want Battery or FuelCell
use sensors::{read_fuel_cell_sensor, read_battery_sensor};
use control::{PidController};
use wasm_bindgen::prelude::*;                    // for #[wasm_bindgen(start)]
use yew::prelude::*;                             // for the Yew framework
use gloo::timers::callback::Interval;            // for periodic updates/ticks

/// A Yew component (the main GUI model).
struct Model {
    fuel_cell: FuelCell,
    battery: Battery,
    pid: PidController,
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
        // Suppose we need a PID for oxygen or SoC control
        let pid = PidController::new(0.3, 0.05, 0.05, 0.5);

        // Example thresholds
        let mut charging_mode = false;
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
            charging_mode,
            cooling_active,
            interval,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                // Place your update logic here:
                // E.g., check if battery SoC is below a threshold => charge
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
                let bat_data = read_battery_sensor(&self.battery);

                // Disturbance
                let disturbance = 10.0; // your custom logic
                // Example humidity & oxygen usage
                let humidity = 0.8;

                // Decide load
                let load = if self.charging_mode {
                    8.0 // charging current
                } else {
                    // Use the pid to track oxygen
                    let pid_output = self.pid.compute(2.0, fc_data.oxygen_concentration);
                    pid_output + disturbance
                };

                // Suppose we set cooling_active if temperature is high
                if self.fuel_cell.temperature > 44.0 {
                    self.cooling_active = true;
                } else {
                    self.cooling_active = false;
                }

                // Update the fuel cell (4 arguments)
                self.fuel_cell.update(
                    load,
                    self.cooling_active,
                    fc_data.oxygen_concentration,
                    humidity
                );

                // Update the battery
                self.battery.update(load * 0.5, load);

                // Return true => re-render if data changed
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // Basic UI in Yew
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
    // Renders our Yew Model to the DOM
    yew::Renderer::<Model>::new().render();
}
