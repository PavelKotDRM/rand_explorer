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

### Code Quality Checks (Rust)

Baseline checks before creating a PR:

```bash
# 1) Check formatting (without changing files)
cargo fmt --all -- --check

# 2) Run lints and fail on warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3) Run tests
cargo test --workspace --all-features

# 4) Build docs and treat warnings as errors
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Quick local fix for formatting:

```bash
cargo fmt --all
```

Security and dependency checks (recommended):

```bash
# Install once
cargo install cargo-audit cargo-deny cargo-outdated cargo-udeps

# 1) Security advisories (RustSec)
cargo audit

# 2) Policy checks: licenses, advisories, bans, sources
cargo deny check

# 3) Outdated dependencies report
cargo outdated --workspace

# 4) Detect unused dependencies
cargo +nightly udeps --workspace --all-targets
```

Note: there is no finite "all possible tools" list in Rust. The commands above cover the most common practical set: style, linting, tests, docs, security, and dependency hygiene.

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
