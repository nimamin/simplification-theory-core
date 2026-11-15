# Vector work coordinate pilot (research only)

The proposed vector addition of kinetic energy and channel work has a necessary caveat: the established kinetic energy `E_k = k_k |v|²` is scalar; work is scalar in classical mechanics as well. One cannot simply add these as physical vectors. A possible **auxiliary encoding**, without changing the scalar energy law, is:

`J_k = k_k |v| v`, so `|J_k| = E_k`; for nonzero `J_k`, `v = J_k / sqrt(k_k |J_k|)`.

In an exploratory seven-case independent Python test, given the already-computed `v' = v + (a_g + a_l)dt`, define the exact vector increment `Delta J = J_k(v') - J_k(v)`. Split it into `Delta J_g = w_g Delta J` and `Delta J_l = w_l Delta J` using the existing nonnegative projection shares with complete cancellation set to zero. **Separately**, compute scalar signed works `W_g=w_g(E_k'-E_k)`, `W_l=w_l(E_k'-E_k)`, and candidate reservoirs `E_g'=E_g-W_g`, `E_l'=E_l-W_l`. Reject nonpositive reservoirs without clamping.

The experiments covered exact cancellation, initial rest, perpendicular acceleration, parallel acceleration, unequal opposition, oblique 3D motion, and reservoir overspend. The vector encoding reconstructs the ALREADY-KNOWN `v'` to floating-point precision and preserves the scalar energy bookkeeping when accepted. The overspend scenario is rejected. The naive differential approximation `Delta J≈2k_k|v|(a_g+a_l)dt` fails to move a particle initially at rest and has large error for perpendicular acceleration.

**This is NOT a new acceleration law:** `Delta J` was calculated from acceleration, not the other way around. It does not yet solve the core user's challenge or confirm that channel work is an independently defined physical vector.

Next: train a dual-particle, variable-field model to predict a vector `Delta J` along with two signed scalar channel works, requiring (1) zero updates on exact cancellation, (2) no direct gravity/levitation transfer, (3) strict positive field reservoirs, (4) conservation, and (5) `|J_k'|=E_k'` without arbitrary clamping. Evaluate holdout full trajectories against finely converged reference solutions. The full Python seven-case pilot, CSV and explanation are delivered in the conversation artifact `escentron_vector_work_pilot.zip` and not included in this repository.
