# Signed channel-work NN pilot (v0.29)

This training experiment switches the two-Escentron macro-step target from **direct final energies** to **signed channel work**:

- `W_g = E_g(t0) - E_g(t1)`
- `W_l = E_l(t0) - E_l(t1)`
- `E_k(t1) = E_k(t0) + W_g + W_l`

Thus no standalone kinetic transfer or direct `E_g <-> E_l` transfer is predicted. A zero-total-acceleration gate forces both signed works to zero when the two effects cancel exactly. Candidate outputs are evaluated in float64 for positive field reservoirs, nonnegative kinetic energy and total energy conservation; **rejection**, not arbitrary floor clamping, is mandatory for invalid suggestions.

Source labels are the previous exploratory two-moving-Escentron, variable-field DOP853 trajectories, split by **whole initial trajectories**: 918 train, 234 validation, 288 test windows (160 distinct trajectories). Each learned work proposal is expressed as a residual correction on the current coarse projection-allocation estimate. The velocity **direction is still calculated from** `v0 + A0*dt`; velocity magnitude follows the work-derived `E_k(t1)`. Therefore this is NOT yet a complete derivation of vector acceleration from work.

| Held-out method | Mean final velocity error | Valid energy proposals |
|---|---:|---:|
| Uncorrected coarse step | 0.029177 | 288/288 |
| First signed-work NN, freer residual | 0.037858 | 266/288 |
| Bounded residual signed-work NN | 0.028512 | 287/288 |
| Validated work NN, coarse-step fallback | **0.028497** | **288/288**, only because the rejected example had a valid fallback |

The validated hybrid provides about **2.3% reduction** in velocity error vs coarse step for this very small exploratory test. Max conservation residual in the updated float64 energy calculation is about 1.78e-15. The residual proposal scale is heuristic; it is not an energy constraint, so the independent rejection mechanism is still essential. On unrelated inputs, the fallback may also be invalid, in which case the correct action is to restart the fine-step reference solver, not to accept an energy violation.

**Caveats:** reference labels are synthetic, not experimentally validated; their per-window timestep convergence has not been independently proven; trajectory holdout does not establish generalization to radically different field regimes. The test uses only 32 held-out trajectories; a 2.3% difference is not yet robust evidence of improvement.

Python training scripts, checkpoint, predictions, comparisons and detailed report were delivered as the conversation artifact `escentron_work_nn/signed_work_nn_pilot.zip`, not committed as binaries to GitHub. Rust tests and CI were not executed as part of this experiment.

**Next:** learn direction changes and signed work jointly, with more diverse nonlinear dynamic-field orbits, strict convergence checks and out-of-distribution field-strength groups. Then attempt compact symbolic regression on the two signed-work functions rather than raw velocity.
