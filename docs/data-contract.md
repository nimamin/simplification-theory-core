# Escentron core: data contract (draft v0.6)

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

## Independent fields (implemented)

At an observation point, each Escentron source produces **two different field types**:

- Gravitational field magnitude: `f_g = G * m / r²` (inward toward the source)
- Levitational field magnitude: `f_l = G * p / r³` (outward from the source)

Their magnitudes are calculated separately and converted into vectors with
their respective directions. **The gravitational and levitational fields are
heterogeneous; they cannot be summed or subtracted to form a net field.**

For multiple source Escentrons, contributions of the **same kind** are
superposed vectorially at each observation point:

- `gravitational_total = sum(gravitational_field_from_each_source)`
- `levitational_total = sum(levitational_field_from_each_source)`

Thus a field query returns **two separate three-dimensional vectors**, not a
single combined field. Vector sums are necessary because contributions of
the same field type from different positions can point in different directions.
The API distinguishes `GravitationalField` and `LevitationalField` as
separate Rust types.

`field_at(source, point, constants)` returns the two field types produced by
one source. `fields_at(sources, point, constants)` sums contributions
separately for all sources at a point.

At a distance `X = p/m` for an isolated source (when `m > 0` and `G > 0`),
the two field **magnitudes** are equal. This is an algebraic equality, **not
a cancellation of heterogeneous fields**, nor a zero acceleration claim.

Evaluation at the exact source position returns an error; zero-distance
softening is not silently introduced. Source fields at a target's own
position must exclude the target itself in any future interaction algorithm.

**Target response is deferred.** The effects of both fields on an Escentron
depend on additional target properties and rules to be specified later. Only
the resulting acceleration contributions may be combined after those rules
are established. No acceleration law, time integration, or particle stability
is implied by field magnitudes alone.

## Target-response software boundary (interface only)

The `ResponseLaw` Rust trait declares two independent operations:

- `from_gravitational(target, gravitational_field) -> acceleration_vector`
- `from_levitational(target, levitational_field) -> acceleration_vector`

No physical implementation of either operation exists yet. A consumer must
supply the response laws explicitly. `acceleration_from_fields` validates
the inputs, evaluates both operations, then adds the **acceleration vectors**.
The software test double is purely an arithmetic/API test, not a physics model.

The response must eventually depend on the relevant target properties and
other components required by the mathematical specification. The current
trait may need extending once these are established.

## Gravitational and levitational response coefficients (partial implementation)

For a target Escentron:

- `E_g = k_g * m`
- `alpha_g = E_g / E_c`
- `a_g_partial = alpha_g * a_g_proposed`
- `E_l = k_l * p`
- `alpha_l = E_l / E_c`
- `a_l_partial = alpha_l * a_l_proposed`

Here `a_g_proposed` is a **separately determined acceleration proposal**,
not the gravitational field itself. The implementation must never equate the
heterogeneous fields with accelerations merely to obtain this proposal.

`gravitational_response_coefficient(target, constants)` calculates the
fraction of the target's cosmic energy stored as gravitational energy.
`scale_gravitational_acceleration(target, constants, proposed)` multiplies
an explicit proposal by that fraction. For zero mass and finite values, the
coefficient is zero. The matching `levitational_response_coefficient(target, constants)`
computes the fraction stored as levitational energy, and
`scale_levitational_acceleration(target, constants, proposed)` applies it to a
separately supplied levitational acceleration proposal. When pulse is zero,
levitational response is zero. Each coefficient rejects a value above the
cosmic energy budget rather than silently clamping it.

This is **only the first factor** of the intended acceleration calculation.
It does not yet model whether the response increases or decreases gravitational
energy, the associated inverse conversion, energy conservation during a step,
or the limit on attainable velocity. The complete gravitational and levitational response laws remain to be specified. The generic
`ResponseLaw` trait remains separate; the new partial coefficient is not
automatically incorporated into every custom implementation.

## Explicitly outside this version

- Full physical laws coupling either field to a target Escentron
- Force, acceleration, integration, and time step
- Per-step energy transfers or redistribution rules
- Numerical stability strategy and collision handling
- Units and numerical calibration
- Composite/complex particles and nuclei
- WASM bindings and visualization

Any of these should be introduced only alongside an explicit updated specification and tests.
