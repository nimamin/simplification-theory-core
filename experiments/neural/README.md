# Energy-constrained neural macro-step (v0.25)

The first working training experiment uses the previous fixed-3D-field synthetic dataset and the currently adopted *projection-cancellation* energy allocation law. It is deliberately separate from the work-based alternative in `work_path.rs`.

The trained PyTorch model uses a cancellation-gated neural residual, then applies an energy-feasible velocity transform and a **mandatory float64 energy validator**. The velocity ceiling is derived from the amount of gravitational or levitational energy that can be spent under the existing fixed pre-step allocation shares. No direct gravity/levitation energy transfer is allowed. The network can propose a candidate; only an accepted candidate may be committed, and rejection must trigger a smaller-step reference computation instead of clamping.

The pilot trained on 3,749 accepted approximate synthetic reference cases (2,248 train, 563 validation, 938 test). Held-out mean Euclidean final velocity error: 0.767639 for conventional single-step, 0.155855 for the constrained NN, and 0.152154 for the safe hybrid. All 938 test NN outputs passed positive-reservoir and conserved-total-energy checks using float64, with maximum total-energy residual 3.55e-15. Cancellation of acceleration vectors is enforced exactly. The 512/1024 step agreement threshold was 0.002, so these are not exact ground-truth targets.

Full reproducible scripts (`train.py`, `infer.py`), input NPZ, trained checkpoint, test NPZ and report were delivered as a conversation ZIP (`escentron_nn_constrained_training.zip`). The GitHub repository contains this report, not the binary checkpoint or dataset. This is **not** a verified result from Rust, not a full physical state integrator, and not proof of orbit stability or generalization to unseen field distributions.

Next: evaluate much stricter convergence-filtered and held-out field-strength groups, and learn a low-cost symbolic expression for the bounded correction.
