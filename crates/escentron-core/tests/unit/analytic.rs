use super::*;

fn approx(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() < tolerance, "{a} != {b}");
}

#[test]
fn exact_gravity_benchmark_matches_independent_python_reference() {
    let state = single_field_constant_exact(0.1, 0.001, 500.0, 0.5, 1.0, 10.0).unwrap();
    approx(state.velocity, 0.10485450879074636, 1e-12);
    approx(state.remaining_field_energy, 5.531986251296731e-6, 1e-12);
    assert!(state.remaining_field_energy > 0.0);
}

#[test]
fn exact_levitation_benchmark_matches_independent_python_reference() {
    let state = single_field_constant_exact(0.1, 9.989, 10.0, 0.5, 1.0, 10.0).unwrap();
    approx(state.velocity, 2.920398420592189, 1e-12);
    approx(state.remaining_field_energy, 1.4702730650026496, 1e-12);
}

#[test]
fn aligned_two_field_energy_is_positive_at_finite_displacement() {
    let s = aligned_fields_energy_at_distance(
        0.001, 9.989, 500.0, 10.0, 0.958, 1.0, 10.0,
    ).unwrap();
    assert!(s.gravitational_energy > 0.0);
    assert!(s.levitational_energy > 0.0);
    assert!(s.gravitational_energy < 0.001);
}

#[test]
fn no_time_passed_preserves_energy_and_velocity() {
    let s = single_field_constant_exact(0.1, 0.001, 500.0, 0.0, 1.0, 10.0).unwrap();
    approx(s.velocity, 0.1, 1e-12);
    approx(s.remaining_field_energy, 0.001, 1e-12);
}

#[test]
fn extremely_long_depletion_fails_explicitly_at_f64_precision_limit() {
    assert_eq!(
        single_field_constant_exact(0.1, 0.001, 500.0, 1e8, 1.0, 10.0),
        Err(AnalyticError::PrecisionLimit)
    );
}
