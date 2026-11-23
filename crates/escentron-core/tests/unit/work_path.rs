use super::*;

#[test]
fn noncollinear_field_energies_follow_independent_dot_products() {
    let s = constant_field_work_path(
        1.0, 8.0, 1.0,
        Vector3::new(2.0, 0.0, 0.0),
        Vector3::new(0.0, 3.0, 0.0),
        Vector3::new(1.0, 1.0, 0.0),
        1.0, 10.0,
    ).unwrap();
    assert!((s.gravitational - (-0.4_f64).exp()).abs() < 1e-12);
    assert!((s.levitational - 8.0 * (-0.6_f64).exp()).abs() < 1e-12);
    assert!((s.gravitational + s.levitational + s.kinetic - 10.0).abs() < 1e-12);
}

#[test]
fn zero_displacement_preserves_all_energies() {
    let s = constant_field_work_path(
        1.0, 8.0, 1.0,
        Vector3::new(3.0, 0.0, 0.0),
        Vector3::new(-2.0, 0.0, 1.0),
        Vector3::ZERO, 1.0, 10.0,
    ).unwrap();
    assert_eq!(s.gravitational, 1.0);
    assert_eq!(s.levitational, 8.0);
    assert_eq!(s.kinetic, 1.0);
}

#[test]
fn inaccessible_reverse_displacement_is_rejected() {
    assert_eq!(
        constant_field_work_path(
            1.0, 8.0, 1.0,
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::ZERO,
            Vector3::new(-5.0, 0.0, 0.0),
            1.0, 10.0,
        ),
        Err(WorkPathError::UnreachableKineticEnergy)
    );
}
