# Symbolic vector regression pilot (v0.26)

**Status: exploratory; no deployable replacement yet.**

The pilot uses the pre-existing 3,749 synthetic *constant-field* samples and the same untouched 938-row held-out test indices from the energy-constrained neural experiment. It searches a **predefined symbolic feature library** with regularized sparse linear regression over rotation-equivariant vector bases. This is *SINDy-like sparse regression*, **not** an unconstrained genetic-programming search and **not** a discovered physical law.

The velocity predictor has the form

\[v_1=v_0+c_A(z)\,A\Delta t+c_D(z)\,(a_g-a_l)\Delta t\,h+c_v(z)\,v_0\,h,\]

where \(A=a_g+a_l\), \(h=\|A\|\Delta t/(1+\|A\|\Delta t)\), and each \(c_i\) is a fitted sum of symbolic scalar features. This construction guarantees **no velocity change under exact cancellation** \(A=0\). Energy validity is NOT guaranteed by the raw equation, so an independent validator must reject outcomes with \(E_g'\le 0\), \(E_l'\le0\), negative kinetic energy, nonfinite values or an energy-budget mismatch. The validator uses the currently adopted **initial-step projection shares**; it does not permit gravity-to-levitation transfers.

Validation-set selection considered Ridge penalties and feature counts (4, 8, 12, 20, 36, 69); the held-out test set was evaluated only after selection.

| Method | Held-out mean velocity L2 error | Accepted by energy checks |
|---|---:|---:|
| Ordinary coarse step | 0.767639 | not all |
| Previous constrained neural predictor | 0.155855 | 938/938 |
| Eight-term symbolic candidate | 0.156377 | 698/938 |
| Twelve-term candidate | 0.145858 | 683/938 |
| Full 69-term candidate | 0.095840 | 703/938 |
| Full symbolic when feasible, constrained NN otherwise | 0.144946 | 938/938, given validated NN fallback |

These are **model-reference errors**, not laboratory measurements. The 69-term fit is not a simple formula. The eight-term fit is about as accurate as the NN in raw error but fails energy validity in ~26% of held-out cases. A global smooth bounded-speed transform made the 69-term formula energy-feasible on the tested holdout but raised its mean velocity error to about **0.309**; it is not an improvement.

**Major model limitation:** around **842/3749 reference targets** lie outside the maximum speed computed from *fixed initial projection shares*. This does not necessarily violate conservation: channel shares evolve during the fine reference trajectory. It does show why a macro-step predictor limited by frozen pre-step shares cannot exactly reproduce all reference endpoints.

Reproducible experimental code, fitted coefficients, inference validator, frozen held-out predictions and visual comparison are supplied as the conversation artifact `escentron_symbolic_first_pass.zip`. That ZIP is not a file in this repository. The Rust physics core was not modified by this pilot, and Rust tests were not executed as part of this experiment.

**Next research step:** learn a *path-integrated* channel-allocation summary or separate bounded energy outcomes directly, respecting zero net acceleration, conservation and strict positive reservoirs; then fit compact symbolic expressions to those constrained targets. Evaluate on genuinely held-out field-strength groups and more strictly converged references.
