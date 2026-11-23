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
fn depleted_first_trial_is_restarted_at_finer_resolution() {
    let options = ConvergenceSettings {
        duration: 1.0, initial_steps: 1, max_steps: 4096,
        absolute_tolerance: 1e-3, relative_tolerance: 1e-3,
    };
    let outcome = convergence_test(&initial(), &constants(), options, |s, _| {
        (Vector3::new(s.mass, 0.0, 0.0), Vector3::ZERO)
    });
    // At N=1 the proposed step would consume more gravitational energy
    // than available. The controller must not report failure at N=1.
    assert!(!matches!(
        outcome,
        Err(ConvergenceError::ReferenceFailed { steps: 1, .. })
    ));
}

#[test]
fn non_depletion_error_still_stops_at_first_resolution() {
    let result = convergence_test(&initial(), &constants(), settings(), |_, _| {
        (Vector3::new(f64::NAN, 0.0, 0.0), Vector3::ZERO)
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
