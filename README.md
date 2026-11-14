# Simplification Theory — Simulation Core

A Rust simulation core for the **Simplification Theory** research project.

The goal is to implement a testable, reproducible computational model of the proposed *Escentron* particle dynamics. This is an exploratory scientific model, not an established description of nature.

## Status

**Data-contract prototype.** Particle state, parameter validation, neutral radius and algebraic energy bookkeeping are implemented. No dynamics or scientific validation is implemented.

## Design goals

- Keep the mathematical model independent from rendering and UI.
- Support native execution and future WebAssembly bindings.
- Make model assumptions, units, constants, and numerical integration methods explicit.
- Add tests and reproducible examples as the model evolves.

## Repository structure

- `crates/escentron-core`: particle state and energy bookkeeping (no dynamics yet).
- `crates/escentron-wasm`: reserved for a future WebAssembly adapter (not yet created).

Read the [data contract](docs/data-contract.md) for definitions, unresolved units and assumptions.\n\n## Build

Install a recent stable Rust toolchain and run:

```sh
cargo test --workspace
```

## Licensing

Copyright © 2026 Nima Amin. All rights reserved.

The code is available under the [PolyForm Strict License 1.0.0](LICENSE). This is **source-available**, not OSI-approved open-source software. The standard license allows specified noncommercial uses but does **not** authorize distribution, changes, derivative works, or commercial use. Those activities require separate permission from the rights holder.

See [LICENSING.md](LICENSING.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
