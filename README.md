# Simplification Theory — Simulation Core

A Rust simulation core for the **Simplification Theory** research project.

The goal is to implement a testable, reproducible computational model of the proposed *Escentron* particle dynamics. This is an exploratory scientific model, not an established description of nature.

## Status

**Initial repository setup.** No physical equations or simulation claims have been implemented or validated yet.

## Design goals

- Keep the mathematical model independent from rendering and UI.
- Support native execution and future WebAssembly bindings.
- Make model assumptions, units, constants, and numerical integration methods explicit.
- Add tests and reproducible examples as the model evolves.

## Repository structure

- `crates/escentron-core`: domain model and simulation logic (currently a minimal placeholder).
- `crates/escentron-wasm`: reserved for a future WebAssembly adapter (not yet created).

## Build

Install a recent stable Rust toolchain and run:

```sh
cargo test --workspace
```

## Licensing

Copyright © 2026 Nima Amin. All rights reserved.

The code is available under the [PolyForm Strict License 1.0.0](LICENSE). This is **source-available**, not OSI-approved open-source software. The standard license allows specified noncommercial uses but does **not** authorize distribution, changes, derivative works, or commercial use. Those activities require separate permission from the rights holder.

See [LICENSING.md](LICENSING.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
