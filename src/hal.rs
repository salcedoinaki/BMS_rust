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
