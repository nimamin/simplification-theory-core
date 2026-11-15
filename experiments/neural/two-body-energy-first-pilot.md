# Variable-field two-body energy-first NN pilot (v0.28)

**This pilot did NOT beat the simple macro-step baseline.**

This experiment uses two moving Escentrons with updated position, gravitational/levitational fields, and evolving mass/pulse at each continuous-ODE evaluation. Initial conditions are randomly varied over mass-pulse asymmetry, separation relative to balance, 3D orientation, and initial velocity. The existing projection-cancellation allocation is used; no direct gravitational-to-levitational transfer or artificial damping is implemented.

- 160 independent initial-condition trajectories / 1,440 training windows
- Grouped by trajectory: 102 training / 26 validation / 32 test trajectories, giving 918 / 234 / 288 windows. No windows from the same trajectory are in both training and test.
- Test mean per-particle Euclidean velocity error: **0.029177** for the uncorrected single coarse step; **0.039071** for the energy-first neural macro-step (34% worse).
- First absolute-energy network was much worse (0.149228 test velocity error); a second residual-parameterized energy-first network improved this to 0.039071.
- Mean absolute channel-energy error of the latter: **0.017453**; mean position-displacement error **0.009836**.
- The new network represents positive channel energies by a softmax and derives speed from the predicted kinetic energy; all 288 held-out windows had positive predicted reservoirs and maximum energy-sum residual around **3.55e-15**. These algebraic constraints alone do not ensure *pathwise* compliance with the no-direct-transfer requirement.
- A gate makes exactly stationary, exactly balanced inputs remain unchanged. Near balance and nonzero initial velocity still require further evaluation.
- Synthetic ODE labels were generated with DOP853 rtol=2e-10 / atol=1e-12 but no independent convergence refinement for each trajectory. These are NOT validated physical ground truth.

### Damping diagnostic

Using the prior 200-time-unit separated-particle trajectories, the initial-separation 11.55 case shows later separation maxima approximately 11.55 and 11.54999, with minima roughly 10.57183, 10.57183 and 10.57182. This gives no resolved monotonic damping trend in the pilot. It cannot answer whether damping would emerge after millions of model-time units, or from different parameters.

Full standalone Python scripts, trained state, trajectory-held-out errors and report are delivered as the conversation artifact `escentron_two_body_energy_first_nn.zip`; this document does not imply that the ZIP was committed to GitHub. Rust compilation or CI checks were not performed.

**Next:** redesign the learned outputs as separate *path-integrated signed channel work* proposals with strict cancellation, physical positivity constraints, and a known fallback; compare against reference-integrated energy allocations over independent initial conditions and much longer windows.
