# Escentron core: data contract (draft v0.23)

This document describes the **data representation and algebraic relations**, not a completed dynamics specification. The equations below describe hypotheses of Simplification Theory; they are not experimentally verified.

## State of one particle

- `position: Vector3` — `x`
- `velocity: Vector3` — `v`
- `mass: f64` — `m`
- `pulse: f64` — `p`

**Existence invariant (theory assumption):** A valid Escentron must satisfy
`m > 0` and `p > 0`. A zero mass or zero pulse is not a special particle
species or a stable boundary state: it contradicts the proposed model of
an existing Escentron. Such input is rejected. The hypothetical disappearance
of a particle at zero is not an implemented transition (and is not treated
as an energy-conserving deletion).

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

`X = p / m` for a valid Escentron, where both `m` and `p` are positive.
Invalid states do not have a neutral radius; no zero-mass or zero-pulse
Escentron exists in the model.

## Validation

The API rejects non-finite inputs, nonpositive mass or pulse, non-finite energy coefficients, and nonpositive `E_c`. An existing Escentron must have strictly positive `m` and `p`. `G` and conversion coefficients may still be zero while their physical restrictions are unresolved. The code rejects overflowed computed energy.

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

**Invalid particle states:** zero mass or zero pulse is disallowed by the
existence invariant, including when both are zero. The field solver does
not create zero-field stand-in particles for either case. Numerical cases
where `X` or `R` underflows/overflows are reported as errors, not treated
as physical disappearance.

The linear inner profile is a modeling choice; no claim is made that it
alone guarantees finite acceleration, total energy conservation, or
numerical stability for arbitrarily large `dt`.

**The basic target response is now specified:** each separate field vector is
scaled by its corresponding target energy fraction, as defined in v0.18 below.
Only the resulting acceleration vectors are summed. This alone does not
establish numerical stability or validate a particular energy-allocation rule.

## Target-response software boundary (interface only)

The `ResponseLaw` Rust trait declares two independent operations:

- `from_gravitational(target, gravitational_field) -> acceleration_vector`
- `from_levitational(target, levitational_field) -> acceleration_vector`

The model now includes `DirectFieldResponse` as its explicitly confirmed
raw field response. Callers can still supply alternative response laws for
comparative software experiments. `acceleration_from_fields` validates
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
an explicit proposal by that fraction. Zero mass is an invalid
Escentron state and is rejected. The matching `levitational_response_coefficient(target, constants)`
computes the fraction stored as levitational energy, and
`scale_levitational_acceleration(target, constants, proposed)` applies it to a
separately supplied levitational acceleration proposal. Zero pulse is likewise
invalid and rejected. Each coefficient rejects a value above the
cosmic energy budget rather than silently clamping it.

This is **the confirmed field-to-acceleration scaling** for the current model.
It does not yet model whether the response increases or decreases gravitational
energy, the associated inverse conversion, energy conservation during a step,
or the limit on attainable velocity. The numerical energy-constrained step remains under investigation. The generic
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
rejects spending more energy than a reservoir possesses, **or exactly
emptying its gravitational or levitational reservoir**. **No energy is
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

## Directional acceleration constraint (geometric helper only)

The author specifies that a coarse discrete timestep may attempt to exhaust
a particle's gravitational or levitational energy, even though the physical
acceleration response should decrease toward depletion. The numerical
correction must **not** scale all components of acceleration: only the
component in the direction responsible for further depletion is restricted.
Perpendicular and reverse components remain active.

The `limit_forward_acceleration(a, d, limit)` helper decomposes an
acceleration with respect to a supplied **depletion direction** `d`:

- `t = (a · d)/(d · d)`
- `a_corrected = a - max(t-limit, 0)*d`

The helper requires no square roots, vector normalization, or trigonometric
functions. With `limit=0`, the forward component is removed while the
transverse and reverse components are preserved.

**Important:** The helper does not determine which acceleration direction
actually depletes a reservoir, the magnitude of its remaining permissible
acceleration, or how two coupled energy channels behave at a depletion event.
Those are physical rules still to be specified. It is NOT yet integrated into
`propose_step`; that function remains a reject-on-overspend diagnostic.

### Potential optimization of the 1,000 microcycle idea

A reference implementation can recompute fields, response coefficients and
reservoir energies in up to 1,000 small substeps to resolve exhaustion.
A faster eventual solver could instead detect the **first reservoir-boundary
event** and split the macro-step there, then continue with the constrained
acceleration. Such an event-driven implementation must recalculate the
fields and target response after the event and must be compared against a
fine-substep reference. Merely clipping mass/pulse or uniformly multiplying
the full acceleration is not equivalent.

This event solver, its coupling to motion, and the choice of strictly
positive numerical minimum for `m` and `p` remain unimplemented.
No claim of unconditional stability for large `dt` is made.

## Restart-on-depletion reference simulation (experimental)

`reference_run(initial, constants, settings, acceleration_at)` is an
experimental single-particle, finite-duration *reference* loop. Its callback
receives the current particle state and time and returns **separate
gravitational and levitational acceleration vectors**. The callback must be
deterministic and must compute each new acceleration from the **updated**
state; no particular field-to-acceleration physics has been assumed.

1. Start from the saved initial state with a specified number of equal
   timesteps and `dt = duration / steps`.
2. On each step, call the acceleration callback, evaluate `propose_step`,
   and (for successful proposals) reconstruct mass and pulse from their
   individual energies: `m'=E_g'/k_g`, `p'=E_l'/k_l`.
   This requires `k_g>0`, `k_l>0`, `k_k>0`.
3. Apply an explicitly provisional semi-implicit position update
   `x'=x+v'*dt`. No physical integration scheme is claimed.
4. If a step would deplete or overdraw either field-specific reservoir
   (or overdraw kinetic energy), **discard the entire trial** and restart
   from the **same initial conditions**, doubling the step count and
   thereby halving `dt`.
5. Stop on the first full successful trial or with a bounded retry-limit
   error once further timestep subdivision would exceed `max_steps`.
   Other errors such as non-finite acceleration terminate immediately.

This does **not** clamp negative energy to zero, invent new energy, mutate
the supplied initial state, or transfer energy directly between the two
non-kinetic reservoirs. Gravitational and levitational energies must remain
strictly positive for every accepted state.

A trial that finishes without overdraw is **not automatically converged**
or physically stable. To build a reliable training reference, compare
the final state and energy trajectory after further decreasing `dt`,
verify energy conservation and inspect sensitivity to the integration
scheme. No neural-network training pipeline has been implemented yet.

The existing field API and response-coefficient helpers are not
automatically wired into this reference runner: the missing complete
response law must still be supplied by its callback. The built-in tests
use synthetic acceleration callbacks to test the *software controller*,
not to validate Simplification Theory.

## Automatic timestep convergence comparison (experimental)

`convergence_test(initial, constants, settings, acceleration_at)` runs
**independent fixed-timestep trials** from the same initial Escentron state
over the same duration. Resolution doubles from `N` to `2N` to `4N`,
continuing up to a caller-specified maximum.

After every pair of valid runs, compare ALL state outputs:
- three position coordinates
- three velocity coordinates
- mass
- pulse

Each individual scalar must satisfy
`abs(new - old) <= absolute_tolerance + relative_tolerance * max(abs(new), abs(old))`.
The returned differences summarize the largest absolute vector-component
errors for position/velocity and the separate mass/pulse errors.

To avoid accepting one accidentally close pair, success requires **two
consecutive refinement pairs** satisfying these criteria. The result reports
the finer reference run, its step count, last differences and number of
comparisons.

Unlike `reference_run`'s retry mode, each individual convergence trial has
a **fixed** timestep: the inner retry limit is set to its trial step count.
If any trial encounters reservoir exhaustion, it is reported immediately
with the resolution that failed. Such a failure is NOT labeled convergence,
nor silently replaced with an unrelated trajectory. For a training-data
workflow, choose a finer initial timestep and restart the entire convergence
comparison.

If the maximum number of steps is reached without two passing pairs,
the routine returns `NotConverged`. This is not an automatic physical
stability proof: agreement at several step sizes can still be misleading
if the supplied acceleration rule is incomplete or a trajectory exhibits
a shared numerical bias.

The acceleration callback must be deterministic and stateless between
reference trials. Meaningful absolute/relative tolerances depend on the
project's eventual physical units. The tests use synthetic acceleration
callbacks and do not validate an Escentron field-response law.

## Connecting external fields to the reference runner (v0.16)

The `coupling.rs` adapter connects previously independent software layers
without inventing a missing field-to-acceleration physical equation:

1. Calculate distinct gravitational and levitational fields at a target's
   current position, excluding that target source by its stable array index.
2. Ask a caller-supplied `ResponseLaw` to convert each field **separately**
   into an **unscaled acceleration proposal**.
3. Apply `alpha_g=E_g/E_c` only to the gravitational proposal and
   `alpha_l=E_l/E_c` only to the levitational proposal, exactly once.
4. Pass the two resulting acceleration vectors to the reference runner.
5. Update the target state after each successful step; the next evaluation
   uses its new position, mass, pulse and velocity.

`reference_with_fixed_sources` supports restart-on-depletion.
`convergence_with_fixed_sources` supports fixed-timestep refinement
comparison. Both evolve **only one target**. Every other Escentron in the
provided source array is **held fixed in position, mass and pulse** over the
trial. This is an intentional reference experiment, NOT a self-consistent
multi-particle simulation.

The `ResponseLaw` implementation is required and cannot be omitted.
It must return *raw* acceleration proposals without applying the energy
fractions itself. In-memory mock laws in unit tests are **software-only**
and are not statements of physics. In particular, merely returning the
field vector as acceleration is not an endorsed field-to-acceleration law.

New checked callbacks in `reference_run_checked` and
`convergence_test_checked` propagate errors in field/response calculation;
they never turn a failed field evaluation into a fabricated acceleration.

**Remaining gap before scientific training data:** the field-to-acceleration
conversion was confirmed and implemented in v0.18, but the energy attribution
and timestep update rules still need validation against the theory specification.
The current pipeline must not be presented as a fully validated physics
solver or used to train a model as if it were established ground truth.

## Confirmed field-to-acceleration model (v0.18)

The theory author confirmed the following model rule for the target Escentron:

- `a_g = (E_g/E_c) * f_g` (the target's gravitational energy fraction
  scales the gravitational field vector)
- `a_l = (E_l/E_c) * f_l` (the target's levitational energy fraction
  scales the levitational field vector)
- `a_total = a_g + a_l` (only acceleration vectors are summed)

The two heterogeneous field types remain separate. No additional
field-to-acceleration coefficient is introduced. This is a **hypothesis
of Simplification Theory**, not an experimentally confirmed law.

`DirectFieldResponse` implements the now-confirmed *raw* proposals by
returning the corresponding field vector; `coupled_acceleration` applies
each energy fraction **once**, outside the raw response law.

The entry points `model_acceleration`,
`reference_model_with_fixed_sources`, and
`convergence_model_with_fixed_sources` use this response directly,
without requiring a mock or a user-supplied response law. The existing
extension points and diagnostic trace remain available.

**Limits:** the reference integrator's energy-allocation rule and
semi-implicit position step are still experimental numerical choices.
Other source particles are held fixed while the single target evolves.
This makes the calculation ready for reproducible model-specific
convergence experiments, not yet a fully validated multi-body model
or a ground-truth data generator for scientific measurements.

## Auditable response trace (v0.17)

The `coupling_trace(sources, target_index, current_target, constants, law)`
function exposes all intermediate quantities required to test a future
field-to-acceleration hypothesis without assuming its formula:

- two separately superposed fields at the target's **current** position,
  excluding the target's own source;
- raw gravitational and levitational acceleration proposals returned by
  the explicitly provided `ResponseLaw`;
- the independent target energy fractions `E_g / E_c` and `E_l / E_c`;
- separately scaled gravitational and levitational acceleration vectors,
  followed by their vector sum.

The `coupled_acceleration` function now shares this same calculation path
and returns the applied acceleration breakdown. The trace is a diagnostic
view of a candidate law, **not** new physics.

### Missing source specification

The Video 4 narrative alone did not give the numerical equation. The
model author subsequently confirmed the direct-field response relation
recorded in v0.18; this question is now resolved for the current model.

The trace tests use software-only mock laws; passing these tests verifies
field separation and energy-fraction wiring, not a scientifically grounded
acceleration formula.

## Reference dataset generation (v0.19, pending CI verification)

The executable Rust example
`crates/escentron-core/examples/generate_reference_dataset.rs` enumerates
36 deterministic initial target states under two fixed, non-collinear external Escentrons,
using provisional numerical values for `G`, energy-conversion factors and
`E_c`. It runs the confirmed field-to-acceleration rule through the
single-target reference solver and the timestep convergence check.

The convergence checker now **automatically refines an initially depleted
trial** (discarding it and restarting from the exact same initial state)
until a valid fixed-step baseline is available, subject to `max_steps`.
It then compares N/2N/4N-like refinements, requiring two consecutive
agreement checks. Other errors are not hidden by refinement.

Only converged cases are printed to **CSV on stdout**, with initial state,
initial energies, separate gravitational/levitational accelerations,
a single coarse-step velocity proposal and whether that proposal passed
the existing energy checks, the reference final state and its energies,
accepted step count, final timestep and four convergence differences.
Failed or nonconverged cases are reported to stderr and omitted from CSV.
This is intentional selection, and such records must not silently be
considered successful training examples.

Run locally with:
`cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
then
`cargo run --release --example generate_reference_dataset > reference-dataset.csv`.

The GitHub Actions workflow is configured to execute these commands and
upload `reference-dataset.csv` as a `reference-dataset` artifact after a
successful run. **The workflow's success has not yet been confirmed.**
No claim that a populated CSV was generated is made until the workflow
actually completes. No neural network or symbolic-regression model has
yet been trained.

These are **synthetic model-consistency** examples, NOT measurements or
experimental confirmation of Simplification Theory. The fixed-source
environment and existing approximate energy allocation may bias training;
the sampler also needs representative depletion-edge conditions before a
learned large-timestep corrector can be assessed.

## Independent numerical pilot and first learning baseline (v0.20)

An independent Python reimplementation of the core arithmetic was
evaluated on 36 initial conditions involving two non-collinear fixed
field sources (so the two independent accelerations are not forced to
be antiparallel). At exploratory absolute and relative tolerances of
1e-4, all 36 cases passed two successive refinement comparisons,
up to 8192 steps. Three initial conditions produced an invalid single
coarse step despite a converged fine-step reference.

This is **not** an execution of the Rust code, nor a Rust CI pass.
The 1e-4 tolerance is deliberately exploratory, and these synthetic
numbers do not imply calibrated physical units. The Python CSV produced
in this separate experiment must not be represented as output of the
Rust example until the Rust executable has actually run.

A group-held-out, four-fold preliminary learning test trained a
regularized linear regression and a small MLP network to predict the
difference between coarse-step and fine-reference final velocities.
Both **performed worse than leaving the coarse velocity unchanged** on
unseen starting-position groups. These negative baseline results are
evidence that 36 hand-picked samples are insufficient for a reliable
learned correction, not evidence for or against the physics hypothesis.

Before symbolic regression, expand the dataset across directions,
velocities, field strengths, and near-depletion cases, and verify
numerical convergence independently of the training split.

## Independent continuous-limit constant-field benchmarks (v0.21)

Single active reservoir, constant signed 1D field `F`, positive kinetic
coefficient `k_k`, and cosmic energy `E_c`:

- `C = E_0 + k_k * v_0^2` is constant.
- `dv/dt = (F/E_c) * (C - k_k*v^2)`.
- Set `U=sqrt(C/k_k)`, `z_0=atanh(v_0/U)`,
  `z(t)=z_0+(F/E_c)*sqrt(C*k_k)*t`.
- Continuous solution: `v(t)=U*tanh(z(t))`;
  `E(t)=C/cosh(z(t))^2`.

The `single_field_constant_exact` Rust function independently implements
this continuous solution without calling `propose_step` or taking any
numerical integration steps. This is the theoretical continuous limit of
the assumed transfer law, not an experimentally established natural law.

For two **co-directed positive constant fields** and 1D forward displacement
`x >= 0`, the continuous allocation rule implies:

- `E_g(x)=E_g(0)*exp(-2*k_k*f_g*x/E_c)`
- `E_l(x)=E_l(0)*exp(-2*k_k*f_l*x/E_c)`

The `aligned_fields_energy_at_distance` function implements the latter
**energy-versus-displacement** invariant; this is not an explicit
position-versus-time solution. The opposing-field case has different
channel attribution and is not covered by the aligned result.

A standalone Python benchmark (`analytic_benchmark.py`) evaluated
the four constant-field test cases. The two single-field formulas
agreed with an independent high-accuracy continuous-time ODE solver
to about 1e-14 in velocity. In the aligned two-field case, the numerical
ODE drove the extremely small gravitational reservoir a few units
below zero at around 1e-15 due to **floating-point solver error**;
the analytic positive exponential evaluates to approximately 2.36e-45
at that position. This is a numerical precision issue, not evidence
of physical negative energy.

IMPORTANT: the earlier constant-field stress-test report marked
**only the gravity-only case converged** under its 1e-8 consecutive-
refinement criterion; the other three reached their test budget before
certified convergence. Their end states can be useful pilot estimates,
but must not be labeled converged under that strict criterion.

The Rust analytic functions, unit tests, and CI compilation have not
yet been confirmed by running Rust in this environment. Both
analytical formulas return an explicit `PrecisionLimit` when floating-
point arithmetic cannot represent the required strictly positive
reservoir; they never silently clamp to a small invented energy.

## Fast aligned two-field numerical experiment (v0.22)

A separate Python prototype now solves the TWO co-directed, constant-field,
positive-velocity, one-dimensional case using the previously derived exact
energy-as-a-function-of-displacement laws:

- `E_g(x)=E_g0*exp(-2*k_k*f_g*x/E_c)`
- `E_l(x)=E_l0*exp(-2*k_k*f_l*x/E_c)`
- `v(x)=sqrt(v0²+(E_g0+E_l0-E_g(x)-E_l(x))/k_k)`
- Find displacement `x` by solving `t=integral(0..x) dx/v(x)`.

The experiment evaluates the time integral by 16-node Gauss-Legendre
quadrature after the *problem-specific* change of variable
`x=s*sinh(u)^2`, where `s=v0²/(2*a0)` and
`a0=(f_g*E_g0+f_l*E_l0)/E_c`. A bracketed scalar root finder then
obtains the final displacement. Both field energies remain positive
for representable exponentials; no depletion clamp is used.

Measured on SIX exploratory cases, the transformed 16-node Python method
was approximately 1.3x to 23.7x faster (median wall-clock) than SciPy
`solve_ivp` with DOP853 and strict tolerances. Its maximum observed
absolute position difference from the ODE was about 4.4e-7 (long
interval), while maximum velocity difference was about 1.2e-8.
The largest errors occur in sensitive regimes; higher quadrature order
improves accuracy but increases work.

These timings are Python-implementation-specific and do NOT predict
Rust or GPU speedups. The quadrature is an **approximation**, and
positive energy does not imply precise position. This method is only
defined for co-directed positive constant fields and a forward-moving
target; opposing, variable, non-collinear and multi-particle fields are
NOT solved by this scheme. It has not been ported to or compiled in Rust.

The independent experiment source and CSV report were produced as
conversation artifacts; this document records the method and its
limitations rather than treating experimental output as verified
core behavior.

## Three-dimensional constant fields: competing reservoir attribution hypotheses (v0.23)

A new independent Python comparison evaluates five constant-field cases:
gravity-dominant opposition, levitation-dominant opposition, orthogonal,
oblique, and parallel-aligned fields. The initial state is common:
`E_c=10`, `k_k=1`, `E_g=0.001`, `v=(0.1,0.04,0)`,
`E_l=10-0.001-|v|²=9.9874`, and duration `T=0.5`.

The CURRENT proposed `allocation.rs` rule attributes *net kinetic change*
according to projected channel acceleration `a_i dot A`, `A=a_g+a_l`.
This is not generally the same as each channel's instantaneous kinetic
work `v dot a_i`. Example:
`a_g=(1,0)`, `a_l=(-2,2)`, `A=(-1,2)`, `v=(1,0)`.
Then `a_g dot A=-1`, resulting in zero assigned gravitational share,
although `v dot a_g=+1`.

ALTERNATIVE, NOT APPROVED AS A REPLACEMENT: attribute energy to the
independent signed kinetic-work terms:
`dE_g/dt=-2*k_k*v dot a_g`,
`dE_l/dt=-2*k_k*v dot a_l`.
Together with `dE_k/dt=2*k_k*v dot (a_g+a_l)`, cosmic energy is conserved.
For *constant* 3D field vectors, using the theory's
`a_i=(E_i/E_c)*f_i`, this produces exact **energy-as-displacement**
relations:
`E_g(x)=E_g(0)*exp[-(2*k_k/E_c)*f_g dot (x-x0)]`,
`E_l(x)=E_l(0)*exp[-(2*k_k/E_c)*f_l dot (x-x0)]`.
They cover opposing and non-collinear constant vector fields as well as
aligned ones. The expressions are positive in exact real arithmetic
for finite displacement but floating-point exponentials may underflow;
such cases must not be silently clamped.

The independent research function `constant_field_work_path`
in `work_path.rs` evaluates this alternate energy relation at a
provided displacement, checking for invalid inputs, numerical
underflow/overflow and impossible negative kinetic energy. **It does
not replace** the canonical `propose_step`, `allocation.rs` or the
reference integrator, and **does not solve x(t) or velocity direction**.
The theory author must decide whether to change the channel-allocation
law before this becomes a model integrator.

The separate Python continuous-ODE tests find strong differences between
the two allocation approaches. For example, in the levitation-dominant
opposing-field case, the current rule leaves `E_g=0.001`, whereas
the independent-work approach yields `E_g≈0.001579` at time 0.5.
These are different **hypotheses**, not numerical errors between two
solvers of the same equations. Numerical tests are exploratory and
not evidence that either hypothesis reproduces nature.

Rust compilation/tests have not yet been verified in this session.

## Explicitly outside this version

- Full physical laws coupling either field to a target Escentron
- Complete acceleration law, energy-constrained correction, and accepted integration step
- Per-step energy transfers or redistribution rules
- Numerical stability strategy and collision handling
- Units and numerical calibration
- Composite/complex particles and nuclei
- WASM bindings and visualization

Any of these should be introduced only alongside an explicit updated specification and tests.
