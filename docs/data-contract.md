# Escentron core: data contract (draft v0.2)

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

## Radial fields (implemented)

At a field observation point displaced by `r_vector` from the source Escentron,
with `r = |r_vector| > 0` and outward unit vector `r_hat`:

- Gravitational magnitude: `f_g = G * m / r²` (inward)
- Levitational magnitude: `f_l = G * p / r³` (outward)
- Net radial field: `f_net = f_l - f_g`
- Net field vector: `F_field = f_net * r_hat`

The capitalization `F_field` is merely a field vector label, **not** a
claim that it is a physical force on a target particle.

For `m > 0`, `G > 0`, and `p > 0`, the net field changes sign at
`X = p/m`: outward for `r < X`, inward for `r > X`. No softening or
arbitrary minimum radius has been introduced; evaluation at `r = 0`
returns an explicit error. Invalid and non-finite inputs and results are rejected.

The field is calculated from the **source** mass and pulse only.
Exactly how a second Escentron responds to this field remains unspecified:
the model needs a target coupling/induction rule before it can calculate
two-body acceleration, time evolution, or stability. A zero of the net
field by itself does not establish a stable atomic orbit.

## Explicitly outside this version

- Coupling between a source field and a target Escentron
- Force, acceleration, integration, and time step
- Per-step energy transfers or redistribution rules
- Numerical stability strategy and collision handling
- Units and numerical calibration
- Composite/complex particles and nuclei
- WASM bindings and visualization

Any of these should be introduced only alongside an explicit updated specification and tests.
