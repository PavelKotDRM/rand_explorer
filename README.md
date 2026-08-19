# rand_explorer

An interactive, GPU-accelerated desktop application written in Rust using egui to explore, visualize, and benchmark the capabilities of the rand crate ecosystem.

---

## Features

* Primitive generation across scalar types (`u8`..`u128`, `f32`, `f64`, `bool`, `char`) with stream and batch modes.
* Range exploration for half-open and inclusive intervals with live validation.
* Statistical distribution sampling with histogram style plots and summary statistics.
* Collection and sequence operations including `choose`, `choose_multiple`, and `shuffle` demonstrations.
* RNG engine switching and micro-benchmarking across deterministic generators.
* Build metadata inspection for Git, compiler, target, and dependency information.

---

## Architecture & Rendering

* GUI Framework: egui / eframe
* Graphics Backend: OpenGL via glow
* Core Engine: rand 0.9+

---

## Getting Started

### Prerequisites

* Rust toolchain (edition 2024 or latest stable)
* C compiler and CMake where required by native dependencies

### Build & Run

```bash
cargo run --release
```

---

## Project Structure

```text
rand_explorer/
├── build.rs
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── app.rs
│   ├── build_info.rs
│   ├── state.rs
│   └── tabs/
├── Cargo.toml
├── README.md
├── LICENSE
└── ТЗ.md
```

---

## License

This project is dual-licensed under the Apache License, Version 2.0 and MIT License.
