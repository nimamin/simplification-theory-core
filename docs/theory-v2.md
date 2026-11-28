# Simplification Theory — Version 2.0

28 November 2026. Based on The Three Escentron Postulates of the same date.
Experimental reference implementation, not a validated physical theory.

## I. Individual cosmic energy

E_c = E_g + E_l + E_k is constant for each particle.

E_g = k_g m^2; E_l = k_l p^2; E_k = k_k |v|^2.

Gravitational mass m and pulse p vary. Inertial mass mu is fixed and k_k = mu/2.

## II. Independent source fields

For displacement from source to receiver r = distance * unit:

F_g = -C_g m_source / distance^2 * unit.
F_l = +C_l p_source / distance^3 * unit.

Sum each field kind independently. Coincidence is undefined in this specification; the v2 engine reports an error instead of introducing a core or collision law.

## III. Work and energy exchange

J = k_k |v| v, |J| = E_k.
Raw work: U_i = k_w E_i F_i dt, where k_w is a positive constant (implemented as 1/F_0).
Use the entire remaining reservoir energy E_i, including the positive minimum.
Applied work W_i is U_i unless the resulting signed transfer would leave a reservoir below E_min.
W = W_g + W_l; J' = J + W; delta E_k = |J'| - |J|.
q_i = (W_i dot W)/|W|^2 * delta E_k; E_i' = E_i - q_i.
If W = 0, q_g = q_l = 0.
Recover m, p and v from their updated energies. No interparticle energy transfer.

## Numerical conventions, not new postulates

The revised work equation uses normalized dt and k_w = 1/F_0. Compute the signed transfers using the raw vectors. If either transfer would spend more than E_i - E_min, apply a common scalar lambda in [0,1] to both raw vectors and recompute the transfers, using a bounded bisection search for a feasible step. This numerical choice preserves the direction and signed shares of the raw resultant; it is not a new physical postulate. A negative transfer replenishes the reservoir and is not constrained by its minimum. Position uses trapezoidal drift; all particles see the same old snapshot. Constants are explicit and units uncalibrated. No artificial rebound or velocity reset is permitted.

An exhausted reservoir retains nonzero raw work because E_min > 0; a reversed field may replenish it. The time symmetry requirement is retained and must be tested, not assumed.

## Acceptance tests

Run cargo test -p escentron-core --test theory_v2. All tests are active; failures are research results, not ignored.

- V2-001: normalized initial m=p=1, v=0, coefficients=1, floor=1e-6, dt=0.001. Apply a constant gravitational field +1 until available energy <=1e-4, then reverse to -1. Require braking time equal to acceleration time within 0.02 normalized time, then negative velocity. Additional exact-floor test uses field magnitude 1000.
- V2-002: identical particles at x=-5,+5, rest, same constants (initial neutral radius 1, initial gap 10). Require initial attraction, approach, rebound without crossing, symmetry and individual energy conservation; repeat dt=0.001, 0.0005, 0.00025 over at most 100 time units. No neutral-radius condition after release. Record first rebound time and minimum gap.
- V2-003: force-free motion.
- V2-004: exact cancellation of work.
- V2-005: individual conservation and floors.
- V2-006: translation and rotation invariance.
- V2-007: refinement at fixed elapsed time.

Tolerances, normalized coefficients and horizons are proposed numerical fixtures, not measured physical constants. The legacy experimental APIs remain available for comparison. The new reference API is escentron_core::theory_v2; it does not yet replace legacy callers. Package version 2.0.0-alpha.2 denotes an unvalidated implementation of theory 2.0.
