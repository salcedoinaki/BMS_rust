mod simulation;
mod sensors;
mod control;
mod hal;

use simulation::{FuelCell, Battery, AirSupplySystem};
use sensors::{read_fuel_cell_sensor, read_battery_sensor};
use control::{OxygenController, AirSupplyController, BatteryController}; // Removed unused PidController import
use wasm_bindgen::prelude::*; // for #[wasm_bindgen(start)]
use yew::prelude::*;          // for Yew components
use gloo::timers::callback::Interval; // for periodic updates
use wasm_bindgen_futures::spawn_local;
use gloo_net::http::Request;
use js_sys::Date;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use log::Level;

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
    interval: Option<Interval>,
    debug_log: Vec<String>, // Accumulated debug output
    simulation_time: f64,   // Elapsed simulation time in seconds
    simulation_duration: f64, // Total simulation duration (e.g., 60 seconds)
}

impl Model {
    /// Sends simulation metrics to InfluxDB using InfluxDB line protocol.
    fn send_metrics(&self) {
        // Get current time in nanoseconds
        let timestamp_ns = (js_sys::Date::now() * 1_000_000.0) as i64;
        
        // Convert booleans to integers (1 for true, 0 for false)
        let charging = if self.charging_mode { 1 } else { 0 };
        let cooling = if self.cooling_active { 1 } else { 0 };
    
        // Create a line of data with multiple fields (uncomment and adjust fields as needed)
        let line = format!(
            "bms_metrics,sim_id=1 \
             voltage={},current={},={},hydration={},
             oxygen={},soc={},battery_voltage={},battery_current={},
             battery_temp={},manifold_pressure={},compressor_speed={},charging_mode={},
             cooling_active={} {}",
            self.fuel_cell.voltage,
            self.fuel_cell.current,
            self.fuel_cell.temperature,
            self.fuel_cell.membrane_hydration,
            self.fuel_cell.oxygen_concentration,
            self.battery.soc,
            self.battery.voltage,
            self.battery.current,
            self.battery.temperature,
            self.air_supply.manifold.pressure,
            self.air_supply.compressor.speed
            charging,
            cooling,
            timestamp_ns
        );
        
        // Print the line for debugging purposes (optional)
        log::debug!("Sending data to InfluxDB: {}", line);
    
        // Use spawn_local to send the HTTP POST asynchronously
        wasm_bindgen_futures::spawn_local(async move {
            // Send the POST request to InfluxDB's write endpoint
            let result = gloo_net::http::Request::post("http://localhost:8086/write?db=bms_db")
                .body(line)
                .send()
                .await;
    
            match result {
                Ok(response) => {
                    if response.ok() {
                        log::debug!("Metrics sent successfully.");
                    } else {
                        log::error!("InfluxDB responded with error: {} {}", response.status(), response.text().await.unwrap_or_default());
                    }
                }
                Err(err) => {
                    log::error!("Failed to send metrics: {:?}", err);
                }
            }
        });
    }
    
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
        let simulation_time = 0.0;
        let simulation_duration = 60.0; // run for 60 seconds

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
            interval: Some(interval),
            debug_log,
            simulation_time,
            simulation_duration,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                let dt = 0.5;
                self.simulation_time += dt;
                
                // Stop simulation after the fixed duration.
                if self.simulation_time >= self.simulation_duration {
                    // Take ownership and cancel the interval.
                    if let Some(interval) = self.interval.take() {
                        interval.cancel();
                    }
                    self.debug_log.push(format!("Simulation ended at {:.2} seconds.", self.simulation_time));
                    return true;
                }
                
                // Update battery mode (hysteresis-based).
                self.charging_mode = self.battery_controller.update_mode(self.battery.soc);

                // Read fuel cell sensor data.
                let fc_data = read_fuel_cell_sensor(&self.fuel_cell);

                // Compute compressor motor torque from AirSupplyController.
                let motor_torque = self.air_supply_controller.compute_motor_torque(fc_data.oxygen_concentration);

                // Estimate mass flow out and update air supply.
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

                // Update battery state.
                if self.charging_mode {
                    self.battery.update(8.0, 0.0, true);
                } else {
                    self.battery.update(0.0, load, false);
                }

                // Append a debug log entry.
                let log_entry = format!(
                    "t: {:.1}s | V: {:.2} V, I: {:.2} A, Temp: {:.2} °C, Hydration: {:.2}, SOC: {:.2}%, MPress: {:.2} Pa, O2: {:.2}",
                    self.simulation_time,
                    self.fuel_cell.voltage,
                    self.fuel_cell.current,
                    self.fuel_cell.temperature,
                    self.fuel_cell.membrane_hydration,
                    self.battery.soc,
                    self.air_supply.manifold.pressure,
                    oxygen_concentration,
                );
                self.debug_log.push(log_entry);
                if self.debug_log.len() > 120 {
                    self.debug_log.drain(0..(self.debug_log.len() - 120));
                }
                self.send_metrics();
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let debug_text = self.debug_log.join("\n");
        html! {
            <div style="font-family: sans-serif;">
                <h1>{ "BMS Simulation (Web) - Debug Output" }</h1>
                <p>{ format!("Simulation Time: {:.1} s / {:.1} s", self.simulation_time, self.simulation_duration) }</p>
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
    console_log::init_with_level(Level::Debug).expect("error initializing logger");
    yew::Renderer::<Model>::new().render();
}
