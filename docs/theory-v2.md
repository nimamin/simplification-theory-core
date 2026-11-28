# Simplification Theory — The Three Escentron Postulates

Version 2.0 · revision alpha.6 · 28 November 2026

## I. Individual cosmic energy

Each Escentron conserves its own cosmic energy:
E_c = E_g + E_l + E_k.
E_g = k_g m²; E_l = k_l p²; E_k = k_k |v|².
Gravitational mass m and pulse p vary. Inertial mass mu is fixed, k_k = mu/2.
m_max = sqrt(E_c/k_g), p_max = sqrt(E_c/k_l) are theoretical single-reservoir limits.

## II. Independent source fields

For unit vector r_hat from source to receiver at distance r > 0:
F_g = -C_g m_source/r² * r_hat; F_l = +C_l p_source/r³ * r_hat.
Sum each field kind independently. Coincidence is undefined.

## III. Field-induced work and energy conversion

Raw work rates:
U_g = k_w (m/m_max) (F_g/F_0).
U_l = k_w (p/p_max) (F_l/F_0).
Here m/m_max = sqrt(E_g/E_c) and p/p_max = sqrt(E_l/E_c).
The ratios are linear in the normalized coordinates, not in normalized energies.
F_0 and k_w are positive reference constants; k_w has energy/time units in the normalized field convention.

Applied step vectors: W_i = lambda U_i dt, with common lambda in [0,1].
W_k = W_g + W_l.
J = k_k |v| v; J' = J + W_k; E_k' = |J'|.
v' = J'/sqrt(k_k |J'|) when J' != 0, otherwise v'=0.
delta E_k = E_k' - E_k.

For W_k != 0, alpha_g = clamp((W_g dot W_k)/(W_k dot W_k), 0, 1);
alpha_l = 1 - alpha_g.
Only uncancelled work participates in energy conversion. For example +6 and -3 yield a resultant +3 and shares 1 and 0.
q_g = alpha_g delta E_k; q_l = delta E_k - q_g.
E_g' = E_g - q_g; E_l' = E_l - q_l.
For exact cancellation, both transfers are zero. No separate braking sign.
Recover m' = sqrt(E_g'/k_g), p' = sqrt(E_l'/k_l).
Total energy is conserved algebraically, up to numerical roundoff.

### Numerical floor and implementation

Use lambda=1 unless a proposed transfer violates E_i' >= E_min > 0. Reduce the common scale by bounded bisection when necessary; preserve the resultant direction and shares. Negative transfers replenish a reservoir. Position uses trapezoidal drift and system steps use a common old snapshot. The reference implementation currently evaluates normalized coordinate ratios as sqrt(E_i/E_c), algebraically equivalent to m/m_max and p/p_max. No artificial rebound or velocity reset. Exact inversion is retained; no direct-coordinate approximation is used in the reference engine. Constants and units remain uncalibrated.

## Acceptance tests

Run cargo test -p escentron-core --test theory_v2. All tests are active; failures are research results, not ignored.

- V2-001: normalized initial m=p=1, v=0, coefficients=1 except k_w=2, floor=1e-6, dt=0.001. Apply a constant gravitational field +1 until available energy <=1e-4, then reverse to -1. Require braking time equal to acceleration time within 0.02 normalized time, then negative velocity. Additional exact-floor test uses field magnitude 1000.
- V2-002: identical particles at x=-5,+5, rest, same constants (initial neutral radius 1, initial gap 10). Require initial attraction, approach, rebound without crossing, symmetry and individual energy conservation; repeat dt=0.001, 0.0005, 0.00025 over at most 100 time units. No neutral-radius condition after release. Record first rebound time and minimum gap.
- V2-003: force-free motion.
- V2-004: exact cancellation of work.
- V2-005: individual conservation and floors.
- V2-006: translation and rotation invariance.
- V2-007: refinement at fixed elapsed time.

Tolerances, normalized coefficients and horizons are proposed numerical fixtures, not measured physical constants. The legacy experimental APIs remain available for comparison. The new reference API is escentron_core::theory_v2; it does not yet replace legacy callers. Package version 2.0.0-alpha.6 is experimental.0.


- V2-008: opposing work cancels before allocation; braking restores only the active reservoir.
- V2-009: aligned work shares and reverse dominance.

## Historical experiments

Alpha.2: user-reported 8/8 tests passed under the earlier signed allocation law.
Alpha.4–5: direct-coordinate integration is retained in the separate experimental module, not the current specification. Its observed energy was not constant. Alpha.6 restores the energy-vector representation and uses cancellation-aware shares. The prior baseline does not establish that alpha.6 passes.
