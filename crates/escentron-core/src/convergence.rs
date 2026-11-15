//! Timestep-refinement convergence diagnostics for the reference solver.
//!
//! Re-run identical initial conditions with N, 2N, 4N ... steps.
//! A run is accepted as numerically converged only after TWO consecutive
//! refinement comparisons satisfy all absolute+relative tolerances.
//! This checks discretization agreement, not physical validity.
use crate::{
    reference_run, ModelConstants, ParticleState, ReferenceError, ReferenceResult,
    RetrySettings, Vector3,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConvergenceSettings {
    pub duration: f64,
    pub initial_steps: usize,
    pub max_steps: usize,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateDifferences {
    pub position: f64,
    pub velocity: f64,
    pub mass: f64,
    pub pulse: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConvergenceResult {
    pub reference: ReferenceResult,
    pub previous_steps: usize,
    pub comparisons: usize,
    pub final_differences: StateDifferences,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConvergenceError {
    InvalidSettings,
    ReferenceFailed { steps: usize, error: ReferenceError },
    NotConverged { finest_steps: usize, final_differences: StateDifferences },
}

impl std::fmt::Display for ConvergenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ConvergenceError {}

fn vector_difference(a: Vector3, b: Vector3) -> f64 {
    (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs())
}

/// Measure the maximum component difference for each state dimension.
pub fn state_differences(a: &ParticleState, b: &ParticleState) -> StateDifferences {
    StateDifferences {
        position: vector_difference(a.position, b.position),
        velocity: vector_difference(a.velocity, b.velocity),
        mass: (a.mass - b.mass).abs(),
        pulse: (a.pulse - b.pulse).abs(),
    }
}

fn close(a: f64, b: f64, abs: f64, rel: f64) -> bool {
    (a - b).abs() <= abs + rel * a.abs().max(b.abs())
}

fn vector_close(a: Vector3, b: Vector3, abs: f64, rel: f64) -> bool {
    close(a.x, b.x, abs, rel)
        && close(a.y, b.y, abs, rel)
        && close(a.z, b.z, abs, rel)
}

fn state_close(a: &ParticleState, b: &ParticleState, abs: f64, rel: f64) -> bool {
    vector_close(a.position, b.position, abs, rel)
        && vector_close(a.velocity, b.velocity, abs, rel)
        && close(a.mass, b.mass, abs, rel)
        && close(a.pulse, b.pulse, abs, rel)
}

/// Run fixed-step reference trials from the SAME initial state, with each
/// resolution twice the previous one. No retry is performed within a trial:
/// a depleted reservoir is reported with its resolution so that the caller
/// can choose a smaller INITIAL timestep and restart the comparison.
///
/// The convergence criterion requires two successive agreeing pairs:
/// N vs 2N and 2N vs 4N. Tolerances refer to raw coordinate/velocity/
/// mass/pulse units; callers must choose units and meaningful tolerances.
///
/// The acceleration callback must be deterministic and stateless across runs.
pub fn convergence_test<F>(
    initial: &ParticleState,
    constants: &ModelConstants,
    settings: ConvergenceSettings,
    acceleration_at: F,
) -> Result<ConvergenceResult, ConvergenceError>
where
    F: Fn(&ParticleState, f64) -> (Vector3, Vector3),
{
    if !settings.duration.is_finite() || settings.duration <= 0.0
        || settings.initial_steps == 0
        || settings.max_steps < settings.initial_steps
        || !settings.absolute_tolerance.is_finite()
        || !settings.relative_tolerance.is_finite()
        || settings.absolute_tolerance < 0.0
        || settings.relative_tolerance < 0.0
        || (settings.absolute_tolerance == 0.0 && settings.relative_tolerance == 0.0)
    {
        return Err(ConvergenceError::InvalidSettings);
    }

    let run = |steps| {
        reference_run(
            initial,
            constants,
            RetrySettings {
                duration: settings.duration,
                initial_steps: steps,
                max_steps: steps,
            },
            &acceleration_at,
        ).map_err(|error| ConvergenceError::ReferenceFailed { steps, error })
    };

    let mut steps = settings.initial_steps;
    let mut previous = run(steps)?;
    let mut previous_pair_passed = false;
    let mut comparisons = 0;
    let mut last_differences = StateDifferences {
        position: 0.0, velocity: 0.0, mass: 0.0, pulse: 0.0,
    };

    while steps <= settings.max_steps / 2 {
        let next_steps = steps * 2;
        let current = run(next_steps)?;
        comparisons += 1;
        last_differences = state_differences(&previous.state, &current.state);
        let passed = state_close(
            &previous.state,
            &current.state,
            settings.absolute_tolerance,
            settings.relative_tolerance,
        );
        if passed && previous_pair_passed {
            return Ok(ConvergenceResult {
                reference: current,
                previous_steps: steps,
                comparisons,
                final_differences: last_differences,
            });
        }
        previous_pair_passed = passed;
        previous = current;
        steps = next_steps;
    }

    Err(ConvergenceError::NotConverged {
        finest_steps: steps,
        final_differences: last_differences,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn constants() -> ModelConstants {
        ModelConstants {
            g: 1.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: 10.0,
        }
    }

    fn initial() -> ParticleState {
        ParticleState {
            position: Vector3::ZERO,
            velocity: Vector3::new(1.0, 0.0, 0.0),
            mass: 6.0,
            pulse: 3.0,
        }
    }

    fn settings() -> ConvergenceSettings {
        ConvergenceSettings {
            duration: 1.0,
            initial_steps: 2,
            max_steps: 128,
            absolute_tolerance: 1e-3,
            relative_tolerance: 1e-3,
        }
    }

    #[test]
    fn zero_acceleration_agrees_across_refinements() {
        let result = convergence_test(&initial(), &constants(), settings(), |_, _| {
            (Vector3::ZERO, Vector3::ZERO)
        }).unwrap();
        assert_eq!(result.reference.accepted_steps, 8);
        assert_eq!(result.comparisons, 2);
        assert_eq!(result.final_differences.mass, 0.0);
        assert_eq!(result.final_differences.pulse, 0.0);
        assert!((result.reference.state.position.x - 1.0).abs() < 1e-12);
    }

    #[test]
    fn weak_constant_acceleration_converges_when_refined() {
        let mut opts = settings();
        opts.max_steps = 4096;
        let result = convergence_test(&initial(), &constants(), opts, |_, _| {
            (Vector3::new(0.1, 0.0, 0.0), Vector3::ZERO)
        }).unwrap();
        assert!(result.reference.accepted_steps >= 8);
        assert!(result.reference.state.mass > 0.0);
        assert!(result.reference.state.pulse > 0.0);
    }

    #[test]
    fn too_tight_tolerance_reports_not_converged() {
        let mut opts = settings();
        opts.max_steps = 8;
        opts.absolute_tolerance = 1e-15;
        opts.relative_tolerance = 1e-15;
        let result = convergence_test(&initial(), &constants(), opts, |_, _| {
            (Vector3::new(0.1, 0.0, 0.0), Vector3::ZERO)
        });
        assert!(matches!(result, Err(ConvergenceError::NotConverged { .. })));
    }

    #[test]
    fn failed_trial_preserves_resolution_and_error() {
        let result = convergence_test(&initial(), &constants(), settings(), |_, _| {
            (Vector3::new(100.0, 0.0, 0.0), Vector3::ZERO)
        });
        assert!(matches!(
            result,
            Err(ConvergenceError::ReferenceFailed { steps: 2, .. })
        ));
    }

    #[test]
    fn invalid_tolerance_is_rejected() {
        let mut opts = settings();
        opts.absolute_tolerance = f64::NAN;
        assert_eq!(
            convergence_test(&initial(), &constants(), opts, |_, _| {
                (Vector3::ZERO, Vector3::ZERO)
            }),
            Err(ConvergenceError::InvalidSettings)
        );
    }

    #[test]
    fn each_vector_component_is_compared() {
        let mut a = initial();
        let mut b = a;
        b.position.z = 0.25;
        b.velocity.y = -0.5;
        a.mass = 6.1;
        let d = state_differences(&a, &b);
        assert_eq!(d.position, 0.25);
        assert_eq!(d.velocity, 0.5);
        assert!((d.mass - 0.1).abs() < 1e-12);
        assert_eq!(d.pulse, 0.0);
    }
}
