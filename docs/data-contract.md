# Escentron core: data contract (draft v0.11)

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

### Finite inner core — provisional regularization

For a source with positive mass and pulse, the inner radius is defined as
`R = INNER_RADIUS_RATIO * X = 0.001 * p/m`. The ratio **0.001** is
a provisional fixed simulation parameter selected by the theory author.
It is not a measured physical constant or a proven bound on stability.

Outside the core (`r >= R`) the original field relations apply.
Within the core (`0 <= r < R`), each field's **magnitude** is its
boundary magnitude multiplied by `r/R`:

- `f_g(r) = (G*m/R²) * (r/R)`
- `f_l(r) = (G*p/R³) * (r/R)`

The two types remain independent. Both vector fields and both magnitudes
are zero at the source center, continuous at the inner boundary, and
finite for every radius when model coefficients and arithmetic remain
within the floating-point range. The inner vectors use the displacement
divided by `R`, avoiding division by `r=0`.

`fields_for_particle(sources, index, constants)` excludes the target's
own field contribution **by index**, including when several particles
occupy the same point. All other sources still contribute. The generic
`fields_at` function evaluates all supplied sources, and is not an
automatic self-excluding query.

**Unresolved edge cases:** `X=p/m` cannot define a positive inner
radius if either `m=0` or `p=0`. Such nonzero sources currently return
`UndefinedCoreRadius` instead of silently restoring a singular field.
A source with both parameters zero produces two zero fields. This is
an implementation boundary, not a proposed physical explanation for
zero-mass or zero-pulse Escentrons.

The linear inner profile is a modeling choice; no claim is made that it
alone guarantees finite acceleration, total energy conservation, or
numerical stability for arbitrarily large `dt`.

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

## Energy exchange between a field-specific reservoir and kinetic energy

The model has three energy reservoirs in each Escentron, with fixed total:

`E_c = E_g + E_l + E_k`.

When a **gravitational** field causes an acceleration that increases kinetic
energy, that increase comes from **the same Escentron's gravitational
energy**:

- `E_g' = E_g - delta_E_k`
- `E_k' = E_k + delta_E_k`
- `E_l' = E_l`

When a **levitational** field causes such an acceleration, the energy comes
from its **levitational energy**:

- `E_l' = E_l - delta_E_k`
- `E_k' = E_k + delta_E_k`
- `E_g' = E_g`

For the reverse conversion (negative `delta_E_k`), kinetic energy returns
to the matching field-specific reservoir. The data transfer operation
rejects spending more energy than a reservoir possesses. **No energy is
transferred between Escentrons.**

The Rust function `transfer_energy(current, source, delta_kinetic)`
implements only this bookkeeping. Its caller specifies both the reservoir
and the signed energy change. It does not decide how strong the acceleration
is, how `delta_E_k` is calculated, how velocities change, or how the two
field effects are reconciled during a step. Any such changes must first be
specified by the theory's full dynamics law.

**Correction of an earlier mistaken assumption:** non-kinetic energies are
*not* fixed during acceleration. The previous `kinetic_budget` calculation
and its `max_speed_with_current_reservoirs` field were removed because they
misrepresented the dynamic energy conversion model. A physically reachable
speed must be derived from the complete transfer and update rules rather
than calculated using frozen reservoirs.

## Allocation of kinetic energy change between the two acceleration channels

This step begins **after** independent gravitational and levitational fields
have generated their respective acceleration vectors. Heterogeneous fields
remain separate throughout the calculation.

Let `a_g` and `a_l` denote the two acceleration vectors, and define
`A = a_g + a_l`. To allocate the effects along the **resultant acceleration
direction**, use dot products rather than computing full projections:

- `d_g = a_g · A`
- `d_l = a_l · A`
- `d_g + d_l = A · A = |A|²`

This identity makes explicit normalization by `|A|`, square roots, angle
calculations and trigonometry unnecessary.

- If `A = 0`: both shares are zero; no kinetic-energy change may be
  attributed to these acceleration contributions.
- If `d_g <= 0`: the gravitational contribution is cancelled along the
  resultant, and levitation receives 100% of the *net* effect.
- If `d_l <= 0`: levitation is cancelled along the resultant, and gravity
  receives 100% of the *net* effect.
- Otherwise both project positively: `w_g = d_g / |A|²` and
  `w_l = 1 - w_g`. The shares add to one.

Given a **separately computed** signed kinetic energy change `delta_E_k`,
assign `delta_E_k_g = w_g * delta_E_k` and
`delta_E_k_l = delta_E_k - delta_E_k_g`.

This means cancelled contributions perform no energy exchange and cannot
directly transfer energy from gravity to levitation or vice versa. The
bookkeeping module `transfer_energy` should be applied independently to
each reservoir with its allocated delta, subject to available energy.

In Rust, `acceleration_shares(a_g, a_l)` and
`allocate_kinetic_change(shares, delta_E_k)` implement these steps.

**Scope:** This allocation is based on projections along the resultant
acceleration. It does not yet model motion-dependent kinetic work, choose
`delta_E_k`, integrate velocity or position, or enforce kinetic-energy
and speed limits. For example, a net acceleration perpendicular to the
current velocity need not have the same kinetic-energy change as a parallel
acceleration. That distinction must be resolved in the dynamics step.

## Finite-step velocity and energy proposal (experimental)

The `propose_step` function evaluates a **candidate**, not an accepted
simulation update. It receives the target state, model constants, already
computed gravitational and levitational acceleration vectors, and a
positive finite time interval `dt`. It does **not** equate heterogeneous
fields with accelerations or infer how a field generates an acceleration.

1. Form total acceleration `A = a_g + a_l`.
2. Propose `v' = v + A * dt` (constant acceleration over the candidate step).
3. Evaluate the **finite-step** change
   `delta_E_k = k_k * (|v'|² - |v|²)`.
   Algebraically this includes `2*k_k*(v·A)*dt + k_k*|A|²*dt²`;
   the quadratic term matters even when velocity and acceleration start
   perpendicular.
4. Allocate that signed change to the two field-specific reservoirs using
   the already defined dot-product acceleration shares.
5. Apply each allocation exclusively between its own non-kinetic reservoir
   and kinetic energy, checking that no reservoir becomes negative.
6. Return the proposed velocity and energy values **without mutating**
   `ParticleState` or assuming a particular integration scheme for position.

The candidate is rejected on invalid step size, invalid initial energy
balance, non-finite arithmetic, unavailable reservoir energy, or mismatched
totals. There is no artificial clamp or implicit energy transfer between
gravity and levitation.

**Important distinction:** the candidate may fail because the proposed
acceleration would overspend its associated energy reservoir. The final
dynamics must decide how to reduce or correct that acceleration, not merely
reject steps. This is still a missing part of the model. In addition,
the projection allocation follows the *resultant acceleration*, not the
direction of velocity; alternative attribution rules based on kinetic
work must not be substituted without an explicit theory decision.

The numerical check currently uses a tolerance of
`1e-10 * max(E_c, 1)` on energy sums. This is an implementation tolerance
for exploratory floating-point tests, **not a physical law**. Tests
and stability studies remain necessary before production simulation use.

## Explicitly outside this version

- Full physical laws coupling either field to a target Escentron
- Complete acceleration law, energy-constrained correction, and accepted integration step
- Per-step energy transfers or redistribution rules
- Numerical stability strategy and collision handling
- Units and numerical calibration
- Composite/complex particles and nuclei
- WASM bindings and visualization

Any of these should be introduced only alongside an explicit updated specification and tests.
