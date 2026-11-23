# Event-resolved zero-velocity transition (2026-11-23)

## Valid initial configuration
Two **identical** free Escentrons, initially at rest and separated by 10, each with `Eg=5, El=5, Ek=0, Ec=10`; `G=50`, all energy coefficients unity. Equal cosmic energy across **all** particles is a mandatory experiment precondition; see `docs/initial-cosmic-energy-invariant.md`.

The previous signed-kinetic-coordinate solver counted tiny alternating signs of relative velocity as hundreds of physical collisions. At velocity zero the proposed **direction-dependent braking correction** has distinct one-sided accelerations and therefore a discontinuous right-hand side. The event-resolved Python pilot locates the **first exact J=0 event** with DOP853 root finding, then checks the one-sided accelerations (toward/away). It **does not** infer physical post-stop behavior from the numerical solver.

In the symmetric 1D reduction `J=|v|v`, `r'= -2 sign(J)sqrt(|J|)`, gravitational field points toward other particle and levitational field away. The branch accelerations at the event are:
`a_plus=C*(f_g*Eg/Ec + f_l)` for inward velocity;
`a_minus=C*(f_g + f_l*El/Ec)` for outward velocity.
For every tested `C`, `a_plus<0<a_minus`: *both* one-sided dynamics drive `J` to zero. A conditional **Filippov sliding** continuation holds the pair at rest, with no artificial oscillation events. **This is an explicit mathematical convention for resolving the velocity-zero discontinuity, not a validated new Simplification Theory axiom.** A different rule/smoothing could produce different long-time motion.

| C | Time of first J=0 | Separation |
|---|---:|---:|
| 1 | 3.52108252 | 2.89349113 |
| 0.1 | 7.06314754 | 1.64028867 |
| 0.01 | 14.80530845 | 1.24073235 |
| 0.001 | 31.67605826 | 1.14007288 |

Refinement with solver maximum steps 0.04/0.02/0.01 yielded first-stop separation variation around 1e-9 to 1e-8. Per-particle conservation residuals were <2e-10; reservoir values were positive. At initially balanced r=1, stationary net acceleration is exactly zero.

**Bug fixed in the separate pilot:** a reservoir that is depleted can still be recharged when its field is braking motion; unlike the old `calc` code, braking coefficient is **1 even when reservoir energy is zero**, while accelerating coefficient remains `E_channel/Ec`. This was checked by a unit test with `Eg=0`, nonzero kinetic energy, and gravitational braking.

The standalone Python research code, six diagnostic output types, numeric tests, and plots are in the conversation artifact `escentron_event_resolved_fix.zip`; they have **not** been integrated or verified in the Rust core. Earlier claims of '1000 real impacts' in a few model-time units are superseded by this numerical diagnosis.

## Eight unresolved research / engineering questions
1. **Physical velocity-zero rule:** should particles remain at rest on the attracting discontinuity, reverse under an explicit physical law, or pass through a smooth transition?
2. **Independent vector channel-work law:** determine each channel's work vector and the scalar energy ledger from field/trajectory without reconstructing them from an assumed acceleration.
3. **Depletion threshold and recovery:** robust event for near-zero reservoirs including recharging and machine precision.
4. **General 3D dynamics:** turns, transverse work, moving non-identical sources, angular momentum / orientation.
5. **Very-close interaction:** derive/test inner-radius regularization and collision/crossing behavior.
6. **Numerical convergence and Rust parity:** establish same trajectories across step sizes and algorithms, and port event solver to the Rust core with tests and CI.
7. **Long-time binding criterion:** genuine amplitude envelope over hundreds/thousands of resolved encounters, distinguishing bounded oscillation from stopping, numerical chatter, or escaping.
8. **Physical parameters/validation:** calibrate conversion coefficients and units and compare predictions with independent measurable phenomena.

This list is a scoped plan, not a statement that no other questions exist.
