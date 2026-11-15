# Vector-work neural pilot (v0.30)

**Hypothesis under test:** gravitational and levitational work are represented by independent 3D vectors whose vector sum gives the *change* of a vector kinetic coordinate. This is a model proposal, not a physical law inferred from measurements.

We choose the auxiliary invertible kinetic coordinate `J=k_k|v|v` (`|J|=E_k`), and train a dual-particle neural model to propose gravitational and levitational channel vectors `B_g, B_l` with `J_next=J+B_g+B_l`. Separate signed scalar channel works `W_g,W_l` must satisfy `W_g+W_l=|J_next|-|J|`, `E_g_next=E_g-W_g>0`, and `E_l_next=E_l-W_l>0`. All proposals must undergo an independent energy validator. Exactly stationary states with exactly canceled accelerations are gated to no movement or energy change.

**Training:** regenerated the established synthetic two-moving-particle, dynamic-field ODE dataset, split by independent trajectories: 918 training windows, 234 validation windows, 288 test windows. No labels are physical measurements, and not every label has undergone strict refinement verification. The ODE source itself still evolves via assumed accelerations and the existing projection-clipped energy allocator; thus this test does *not* derive acceleration from work independently.

**Held-out mean per-particle L2 velocity error:** coarse single step `0.0291772`; best restricted vector-work model `0.0299420` (**2.6% worse**). Larger 0.08-scale correction yielded `0.0320163` and was worse. The smaller residual scale was selected using validation results. All 288 vector-work test predictions had strictly positive gravitational and levitational energies; maximal energy-sum discrepancy was `1.78e-15`, and vector-addition discrepancy `2.78e-17`. These are imposed structural identities, not learned physical evidence.

The trained PyTorch checkpoint, reference-data regeneration code, network training code, CSV holdout predictions, chart and full experiment README are in the conversation artifact `escentron_vector_work_neural_pilot.zip` (not committed as binary files to GitHub). Rust core was not modified and Rust tests were not run for this pilot.

**Next:** formalize what a *physical vector channel work* means independently of acceleration; obtain independently verified trajectory-level references with actual changing fields, and test whether the vector-work model predicts motions across unseen initial distances, energies, and source strengths. Keep vector-kinetic magnitude and scalar reservoir bookkeeping consistent without silently allowing direct field-reservoir transfer.
