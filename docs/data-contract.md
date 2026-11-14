# Escentron core: data contract (draft v0.1)

This document describes the **data representation and algebraic relations**, not a completed dynamics specification. The equations below describe hypotheses of Simplification Theory; they are not experimentally verified.

## State of one particle

- `position: Vector3` — `x`
- `velocity: Vector3` — `v`
- `mass: f64` — `m`
- `pulse: f64` — `p`

Position and velocity are three-dimensional. Acceleration is **not stored** as an independent state variable. The data contract does not yet define a coordinate system, units or a time integrator.

## Model constants (must be supplied explicitly)

- `g` — `G`, shared field scaling coefficient
- `k_g` — gravitational energy conversion coefficient
- `k_l` — levitational energy conversion coefficient
- `k_k` — kinetic energy conversion coefficient
- `cosmic_energy` — `E_c`, shared cosmic energy per particle

Numeric values and final unit conventions are **not yet selected**. No built-in values are assumed.

## Energy decomposition

- `E_g = k_g * m`
- `E_l = k_l * p`
- `E_k = k_k * (v_x² + v_y² + v_z²)`
- Model conservation claim: `E_g + E_l + E_k = E_c`

The Rust code computes the three energy terms and a diagnostic residual:
`residual = E_g + E_l + E_k - E_c`.

It does not force conservation, modify particle parameters or perform integration.

## Neutral radius

`X = p / m` when `m > 0`.

For `m = 0`, the method returns `None` rather than introducing an assumed infinity, special radius, or photon rule. The model's treatment of this limit remains open.

## Validation

The initial API rejects non-finite inputs, negative mass/pulse, non-finite energy coefficients, and nonpositive `E_c`. Zero mass or pulse remains representable. `G` and conversion coefficients may be zero while their physical restrictions are unresolved. The code rejects overflowed computed energy.

These are conservative numerical input checks, **not proofs of physical validity**. Extreme finite position/velocity values may still overflow subsequent calculations.

## Explicitly outside this version

- Field formulas and their interactions
- Force, acceleration, integration, and time step
- Per-step energy transfers or redistribution rules
- Numerical stability strategy and collision handling
- Units and numerical calibration
- Composite/complex particles and nuclei
- WASM bindings and visualization

Any of these should be introduced only alongside an explicit updated specification and tests.
