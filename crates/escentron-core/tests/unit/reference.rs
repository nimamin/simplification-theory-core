use super::*;

fn constants() -> ModelConstants {
    ModelConstants {
        g: 1.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: 10.0,
    }
}
fn state() -> ParticleState {
    ParticleState {
        position: Vector3::ZERO,
        velocity: Vector3::new(1.0, 0.0, 0.0),
        mass: 6.0,
        pulse: 3.0,
    }
}
fn settings() -> RetrySettings {
    RetrySettings { duration: 1.0, initial_steps: 1, max_steps: 8192 }
}

#[test]
fn retries_from_original_state_when_coarse_step_depletes() {
    // Test-only acceleration with gravitational response proportional
    // to the remaining source reservoir; not the theory's full field law.
    let result = reference_run(&state(), &constants(), settings(), |s, _| {
        (Vector3::new(s.mass, 0.0, 0.0), Vector3::ZERO)
    }).unwrap();
    assert!(result.retries > 0);
    assert!(result.accepted_steps > 1);
    assert!(result.state.mass > 0.0);
    assert!(result.state.pulse > 0.0);
    let e = energy_of(&result.state, &constants()).unwrap();
    assert!((e.total() - 10.0).abs() < 1e-9);
}

#[test]
fn a_stable_trial_needs_no_retry() {
    let result = reference_run(&state(), &constants(), settings(), |_, _| {
        (Vector3::ZERO, Vector3::ZERO)
    }).unwrap();
    assert_eq!(result.retries, 0);
    assert_eq!(result.accepted_steps, 1);
    assert_eq!(result.state.mass, 6.0);
    assert_eq!(result.state.pulse, 3.0);
}

#[test]
fn retries_have_a_finite_limit() {
    let options = RetrySettings { duration: 1.0, initial_steps: 1, max_steps: 2 };
    let result = reference_run(&state(), &constants(), options, |_, _| {
        (Vector3::new(100.0, 0.0, 0.0), Vector3::ZERO)
    });
    assert!(matches!(result, Err(ReferenceError::RetryLimitReached { .. })));
}

#[test]
fn other_invalid_steps_do_not_trigger_retries() {
    let result = reference_run(&state(), &constants(), settings(), |_, _| {
        (Vector3::new(f64::NAN, 0.0, 0.0), Vector3::ZERO)
    });
    assert!(matches!(result, Err(ReferenceError::Step(_))));
}

#[test]
fn rejects_zero_coefficient_for_energy_to_mass_conversion() {
    let mut c = constants();
    c.k_g = 0.0;
    assert_eq!(
        reference_run(&state(), &c, settings(), |_, _| {
            (Vector3::ZERO, Vector3::ZERO)
        }),
        Err(ReferenceError::UnsupportedConversionCoefficient)
    );
}
