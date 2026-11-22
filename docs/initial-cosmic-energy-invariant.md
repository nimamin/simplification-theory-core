# Mandatory experiment invariant: equal initial cosmic energy

**Author's binding requirement (2026-11-22):** In every valid Simplification Theory simulation, all Escentrons **must begin with exactly the same cosmic energy** `E_c`, even when their initial gravitational, levitational, and kinetic energy allocations differ.

For every particle `i`: `E_g_i(0) + E_l_i(0) + E_k_i(0) = E_c`, with one **shared** `E_c` for all particles. Do not compare or use experimental results from a setup that violates this invariant. Validate it programmatically before simulation. Subsequent total energy must be individually conserved for each particle, subject to explicit numerical precision handling. The author explicitly requested this requirement never be forgotten.

## Current identical-particle comparison

Two *identical* Escentrons at rest: each `E_g=5, E_l=5, E_k=0, E_c=10`; initial separation `r=10`, `G=50, k_g=k_l=k_k=1`. For these two particles, initial neutral distance is `r_eq = E_l1*E_l2/(E_g1*E_g2) = 1`.

Independent Python test of old always-reservoir correction vs proposed velocity-direction-dependent braking correction with work coefficient `C=1,0.1,0.01,0.001` and `dt=0.005`; directional also checked at `dt=0.0025`. At `C=0.1`, old model separation maxima rise from 10.149 to 12.192 in the first twelve cycles, while directional model contracts to `r≈1.736` and stays there over many *numerically detected* close-turning events. However, near this point very many tiny sign flips are counted as collisions (1000 in ~22 simulated time units), a serious **chattering / discontinuous direction-switching artifact**. Thus these are **not confirmed 1000 physical oscillations** or proof of stable binding. With C=0.01 the two dt values yield noticeably different extrema; improving smoothness and convergence is required. With `C=1`, old model displayed slightly *decreasing* maxima near r=10 over 29 observed maxima, so the assertion 'old model always grows' is false.

The Python code, CSV data, and plot are delivered in conversation artifact `escentron_equal_identical_particles.zip`; this document does not imply GitHub houses the ZIP or that Rust CI passed.

Next: event-driven crossing through zero velocity, eliminate numerical chattering, validate strict convergence and monitor changes in kinetic and field-reservoir energies before interpreting orbit/binding trends.
