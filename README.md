# PEM Fuel Cell Simulator with Control

A modular Rust-based simulator for a PEM (Proton Exchange Membrane) fuel cell system. This project models the dynamic behavior of a fuel cell integrated with a battery, air supply system (compressor and manifold), and hardware abstraction for sensor and actuator simulation. It also implements control strategies (e.g., adaptive oxygen regulation and thermal management) to mimic real-world operation.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Project Structure](#project-structure)
- [Installation & Build Instructions](#installation--build-instructions)
- [Usage](#usage)
  - [Console Mode](#console-mode)
  - [Web Mode](#web-mode)
- [Simulation Details](#simulation-details)
  - [Fuel Cell Model](#fuel-cell-model)
  - [Battery Model](#battery-model)
  - [Air Supply System](#air-supply-system)
- [Control Strategies](#control-strategies)
- [Testing](#testing)
- [Assumptions & Modeling Simplifications](#assumptions--modeling-simplifications)
- [Cheat Sheets & Notes](#cheat-sheets--notes)
- [Future Work](#future-work)
- [License](#license)

---

## Introduction

This project simulates a PEM fuel cell system, integrating multiple subsystems such as the fuel cell itself, a battery model, and the supporting air supply mechanism. It uses simplified physical models for dynamics (electrochemical, thermal, and fluid flow) and control loops to mimic real-world conditions. The simulator is implemented in Rust and can be run in both console and web (Yew) interfaces.

---

## Features

- **Fuel Cell Simulation:** Dynamic model including polarization losses (activation, ohmic, and concentration losses) and temperature dynamics with membrane hydration effects.
- **Battery Model:** Updates state of charge (SoC), voltage, and temperature based on charging/discharging cycles.
- **Air Supply System:** Simplified compressor and manifold models that compute mass flow rates and update manifold pressure using mass balance and proportional control.
- **Control Logic:** Adaptive oxygen control and thermal management to optimize system performance.
- **Hardware Abstraction:** Simulated sensors (temperature, voltage, current, etc.) and actuators to emulate real hardware interactions.
- **Dual Interface:** 
  - **Console Application:** A simulation loop running for a defined number of steps with debug outputs.
  - **Web Interface:** (Using Yew framework) provides a graphical debug display.

---

## Project Structure

```
src/
├── control.rs         # Implements the oxygen controller for adaptive regulation.
├── hal.rs             # Hardware abstraction layer for sensors and actuators.
├── lib.rs             # Central library file tying modules together.
├── main_console.rs    # Console-based simulation loop and main entry point.
├── sensors.rs         # Simulated sensor modules for fuel cell and battery.
├── simulation.rs      # Core simulation models and submodules.
│   ├── compressor.rs  # Compressor model for the air supply subsystem.
│   └── manifold.rs    # Manifold model for pressure management.
└── TODO.txt           # Notes for future improvements and documentation tasks.
```

---

## Installation & Build Instructions

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)
- (For web interface) [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) and basic web development tools

### Building the Console Application

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/PEM-Fuel-Cell-Simulator.git
   cd PEM-Fuel-Cell-Simulator
   ```

2. Build and run the console simulation:

   ```bash
   cargo run --bin main_console
   ```

### Building the Web Interface

1. Build the WebAssembly target:

   ```bash
   wasm-pack build --target web
   ```

2. Serve the generated files (for example, using a simple HTTP server):

   ```bash
   python -m http.server 8000
   ```

3. Navigate to `http://localhost:8000` in your browser to view the web debug interface.

---

## Usage

### Console Mode

- **Execution:** The console application (`main_console.rs`) runs a simulation loop (default 100 steps).
- **Behavior:** At each time step, the simulation updates:
  - Fuel cell state (voltage, current, temperature, hydrogen flow, oxygen concentration).
  - Battery state (SoC, voltage, current, temperature).
  - Air supply subsystem including compressor speed and manifold pressure.
  - Cooling is activated based on temperature thresholds.
  - The oxygen controller adjusts load to maintain optimal oxygen concentration.
- **Debug Output:** Each simulation step prints the current state of the fuel cell and battery.

### Web Mode

- **Interface:** A web application built with the Yew framework displays live debug information.
- **Displayed Metrics:** Voltage, current, temperature, membrane hydration, manifold pressure, oxygen concentration, battery SoC, and actuator states.
- **Debug Log:** A scrolling debug log shows recent simulation events.

---

## Simulation Details

### Fuel Cell Model

- **Core Variables:**
  - **Voltage & Current:** Determined by an open-circuit voltage reduced by activation, ohmic, and concentration overpotentials.
  - **Hydrogen Flow:** Scales with load.
  - **Temperature Dynamics:** Increases with load-induced heat and decreases with active cooling.
  - **Membrane Hydration:** Updated using a first-order differential equation. Affects effective resistance.
  - **Oxygen Concentration:** Computed based on manifold pressure; lower values reduce effective voltage.

- **Equations:**
  - *Effective OCV:* Adjusted by a temperature coefficient.
  - *Activation Loss:* Logarithmic function of load relative to the exchange current.
  - *Ohmic Loss:* Load multiplied by effective resistance (inversely related to hydration).
  - *Concentration Loss:* A logarithmic function capped when load exceeds a limiting current.
  
### Battery Model

- **State Variables:**
  - **State of Charge (SoC):** Updated based on net charging/discharging current.
  - **Voltage:** Computed from an open-circuit voltage that depends on SoC with a quadratic relationship.
  - **Temperature:** Updated implicitly in the simulation.
  
- **Charge/Discharge Logic:**
  - Switching between charging and discharging is based on SoC thresholds (65% for charging, 75% for discharging).

### Air Supply System

- **Compressor:**
  - **Dynamics:** Updates rotational speed based on motor torque input and load torque.
  - **Mass Flow Rate:** Computed with an exponential decay function with respect to the pressure ratio.
  - **Load Torque:** Proportional to the mass flow rate.
  
- **Manifold:**
  - **Pressure Update:** Uses mass balance (inflow minus outflow), leakage, and active venting when pressure exceeds a target (4 bar).
  - **Control Terms:** Includes a proportional control term to reduce pressure when above target, with adjustments based on the discharging mode.

---

## Control Strategies

- **Oxygen Controller:** Implements adaptive control to maintain the optimal oxygen concentration at the fuel cell inlet. Adjusts the load using a regulation function that factors in the current oxygen concentration and a disturbance term.
- **Thermal Management:** A hardware abstraction layer monitors temperature via a simulated sensor. When the temperature exceeds 44 °C, a simulated actuator is activated to initiate cooling, with the cooling efficiency altering the fuel cell’s temperature response.
- **Charging Mode Switching:** The simulation monitors battery SoC to switch between charging (when SoC is low) and discharging (when SoC is high).

---

## Testing

The project includes unit tests for key components:

- **Sensors:** Tests for ensuring sensor reading functions return correct data for the fuel cell and battery.
- **Simulation:** Tests to verify that the fuel cell’s temperature response differs when cooling is active versus inactive, and that the battery updates its SoC appropriately.

Run the tests with:

```bash
cargo test
```

---

## Assumptions & Modeling Simplifications

- **Fuel Cell:** Voltage drops are calculated as a sum of activation, ohmic, and concentration losses using simplified empirical formulas. Membrane hydration affects the effective resistance.
- **Battery:** The model uses a simple fixed internal resistance and a quadratic dependency for open-circuit voltage.
- **Air Supply:** Compressor and manifold dynamics are represented with simplified exponential and proportional control relationships, rather than full fluid dynamics.
- **Control Parameters:** The simulation uses fixed parameters (e.g., dt, cooling thresholds, SoC limits) chosen to capture qualitative system behavior without detailed calibration against experimental data.

---

## Cheat Sheets & Notes

### Key Parameters

| Component         | Parameter                          | Typical Value         | Notes                                         |
|-------------------|------------------------------------|-----------------------|-----------------------------------------------|
| **Fuel Cell**     | Base OCV                           | 60.0 V                | Adjusted by temperature coefficient         |
|                   | Internal resistance (`r_internal`) | 0.1 Ω                 | Modulated by membrane hydration             |
|                   | Membrane hydration                 | 1.0 (max)             | Bounds: 0.1 - 1.0, updated with dt = 0.5 s      |
|                   | Limiting current                   | 1.5 A                 | Used for concentration loss calculation       |
| **Battery**       | SoC                                | 100% (initial)        | Updated with net current (charge/discharge)   |
|                   | Voltage scaling (OCV)              | 47.0 V base + 6.0 V    | Quadratic dependency on SoC                  |
| **Air Supply**    | Manifold target pressure           | 380000 Pa (4 bar)     | Includes leak and vent control terms          |
|                   | Compressor inertia                 | 0.1 kg·m²             | Affects dynamic response                      |

### Simulation Cycle Overview

1. **Read Sensors:** Fuel cell and battery sensor functions provide current measurements.
2. **Control Decisions:** Oxygen controller and battery SoC thresholds decide on load adjustment and mode switching.
3. **Subsystem Updates:**
   - **Fuel Cell:** Updates voltage, temperature, hydrogen flow, and hydration.
   - **Battery:** Adjusts SoC, voltage, and current.
   - **Air Supply:** Compressor speed and manifold pressure updated based on mass flow calculations.
4. **Hardware Abstraction:** Temperature sensor reading triggers actuator for cooling if necessary.
5. **Logging & Debug Output:** State information printed (console) or displayed (web interface).

---

## Future Work

- **Enhanced Modeling:** Incorporate more detailed fluid dynamic models and experimental calibration.
- **Expanded Control Strategies:** Implement advanced PID or model predictive control (MPC) for oxygen and thermal management.
- **Web UI Enhancements:** Improve real-time plotting and interactive simulation controls.
- **Documentation Improvements:** Continue expanding inline documentation and developer notes as the model evolves.

---

## License

This project is licensed under the [MIT License](LICENSE).

---

Happy simulating!
