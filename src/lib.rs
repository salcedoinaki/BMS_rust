use gloo::timers::callback::Interval;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod simulation;
mod sensors;
mod control;
mod hal;

// Bring simulation functions into scope.
use simulation::{Battery, FuelCell};
use sensors::{read_battery_sensor, read_fuel_cell_sensor};
use control::PidController;
use hal::{HardwareInterface, SimulatedActuator, SimulatedTemperatureSensor};

struct Model {
    fuel_cell: FuelCell,
    battery: Battery,
    pid: PidController,
    charging_mode: bool,
    cooling_active: bool,
    interval: Interval,
}

enum Msg {
    Tick,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let fuel_cell = FuelCell::new();
        let battery = Battery::new();
        let pid = PidController::new(70.0, 0.3, 0.05, 0.05, 0.5);

        let link = ctx.link().clone();
        let interval = Interval::new(500, move || link.send_message(Msg::Tick));

        Self {
            fuel_cell,
            battery,
            pid,
            charging_mode: false,
            cooling_active: false,
            interval,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        let lower_threshold = 65.0;
        let upper_threshold = 75.0;
        let charging_current = 8.0;

        // Determine charging/discharge mode based on battery SoC.
        if self.charging_mode {
            if self.battery.soc > upper_threshold {
                self.charging_mode = false;
            }
        } else {
            if self.battery.soc < lower_threshold {
                self.charging_mode = true;
            }
        }

        if self.charging_mode {
            // In charging mode, update the simulation accordingly.
            self.fuel_cell.update(charging_current, self.cooling_active);
            self.battery.update(charging_current, 0.0);
        } else {
            // In discharge mode, compute a load via the PID controller.
            let disturbance = 10.0; // simplified fixed disturbance
            let load = self.pid.compute_load(self.battery.soc, disturbance);
            self.fuel_cell.update(load, self.cooling_active);
            self.battery.update(load * 0.5, load);
        }

        // Simulate cooling: if fuel cell temperature exceeds 44°C, activate the cooling fan.
        self.cooling_active = self.fuel_cell.temperature > 44.0;

        true // trigger re-render
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let fc = &self.fuel_cell;
        let bat = &self.battery;
        html! {
            <div style="font-family: sans-serif;">
                <h1>{ "BMS Simulation (Web)" }</h1>
                <p>{ format!("Fuel Cell: Voltage: {:.2} V, Current: {:.2} A, Temp: {:.2} °C", fc.voltage, fc.current, fc.temperature) }</p>
                <p>{ format!("Battery: SoC: {:.2} %, Voltage: {:.2} V, Current: {:.2} A, Temp: {:.2} °C", bat.soc, bat.voltage, bat.current, bat.temperature) }</p>
                <p>{ format!("Cooling Active (Fan): {}", if self.cooling_active { "Yes" } else { "No" }) }</p>
                <p>{ format!("Charging Mode: {}", if self.charging_mode { "Yes" } else { "No" }) }</p>
            </div>
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    html! { <Model /> }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::start_app::<Model>();
}
