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
A_i = E_i - E_min.
U_i = A_i F_i/F_0.
W_i = A_i F_i/max(F_0, |F_i|).
W = W_g + W_l; J' = J + W; delta E_k = |J'| - |J|.
q_i = (W_i dot W)/|W|^2 * delta E_k; E_i' = E_i - q_i.
If W = 0, q_g = q_l = 0.
Recover m, p and v from their updated energies. No interparticle energy transfer.

## Numerical conventions, not new postulates

The original work equation specifies a step, not a physical rate. For the reference implementation only, use normalized dt: W_i = A_i * unit(F_i) * min(dt |F_i|/F_0, 1). At dt=1 this is the documented work rule. Position uses trapezoidal drift; all particles see the same old snapshot. Constants are explicit and units uncalibrated. No artificial rebound or velocity reset is permitted.

Exact saturation is an absorbing state for an exhausted channel under the documented rule, even if its field reverses. This conflicts with the requested acceptance test; do not hide the conflict or modify the postulates silently.

## Acceptance tests

Run cargo test -p escentron-core --test theory_v2. All tests are active; failures are research results, not ignored.

- V2-001: normalized initial m=p=1, v=0, coefficients=1, floor=1e-6, dt=0.001. Apply a constant gravitational field +1 until available energy <=1e-4, then reverse to -1. Require braking time equal to acceleration time within 0.02 normalized time, then negative velocity. Additional exact-floor test uses field magnitude 1000.
- V2-002: identical particles at x=-1,+1, rest, same constants. Require approach, entry inside instantaneous neutral radius, rebound without crossing, symmetry and individual energy conservation; repeat dt=0.001, 0.0005, 0.00025 over at most 40 time units.
- V2-003: force-free motion.
- V2-004: exact cancellation of work.
- V2-005: individual conservation and floors.
- V2-006: translation and rotation invariance.
- V2-007: refinement at fixed elapsed time.

Tolerances, normalized coefficients and horizons are proposed numerical fixtures, not measured physical constants. The legacy experimental APIs remain available for comparison. The new reference API is escentron_core::theory_v2; it does not yet replace legacy callers. Package version 2.0.0-alpha.1 denotes an unvalidated implementation of theory 2.0.
