use super::*;

fn v(x: f64, y: f64) -> Vector3 { Vector3::new(x, y, 0.0) }
fn eq(a: f64, b: f64) { assert!((a - b).abs() < 1e-12, "{a} != {b}"); }

#[test]
fn aligned_accelerations_share_by_projected_effect() {
    let s = acceleration_shares(v(3.0, 0.0), v(1.0, 0.0)).unwrap();
    eq(s.gravitational, 0.75);
    eq(s.levitational, 0.25);
    assert_eq!(s.total, v(4.0, 0.0));
    assert_eq!(allocate_kinetic_change(s, 20.0).unwrap(), (15.0, 5.0));
}

#[test]
fn opposing_collinear_acceleration_is_cancelled() {
    let s = acceleration_shares(v(4.0, 0.0), v(-2.0, 0.0)).unwrap();
    assert_eq!(s.total, v(2.0, 0.0));
    assert_eq!((s.gravitational, s.levitational), (1.0, 0.0));
    assert_eq!(allocate_kinetic_change(s, 10.0).unwrap(), (10.0, 0.0));
}

#[test]
fn weaker_gravity_is_cancelled_by_levitation() {
    let s = acceleration_shares(v(-1.0, 0.0), v(3.0, 0.0)).unwrap();
    assert_eq!((s.gravitational, s.levitational), (0.0, 1.0));
}

#[test]
fn angled_positive_projections_split_energy() {
    let s = acceleration_shares(v(2.0, 0.0), v(0.0, 2.0)).unwrap();
    eq(s.gravitational, 0.5);
    eq(s.levitational, 0.5);
}

#[test]
fn angled_opposing_projection_gets_no_share() {
    let s = acceleration_shares(v(3.0, 0.0), v(-2.0, 1.0)).unwrap();
    assert_eq!(s.total, v(1.0, 1.0));
    assert_eq!((s.gravitational, s.levitational), (1.0, 0.0));
}

#[test]
fn perfect_cancellation_has_zero_shares() {
    let s = acceleration_shares(v(2.0, 1.0), v(-2.0, -1.0)).unwrap();
    assert_eq!((s.gravitational, s.levitational), (0.0, 0.0));
    assert_eq!(allocate_kinetic_change(s, 0.0).unwrap(), (0.0, 0.0));
    assert_eq!(allocate_kinetic_change(s, 1.0), Err(AllocationError::NonFiniteInput));
}

#[test]
fn reverse_kinetic_change_retains_allocation() {
    let s = acceleration_shares(v(1.0, 0.0), v(3.0, 0.0)).unwrap();
    assert_eq!(allocate_kinetic_change(s, -8.0).unwrap(), (-2.0, -6.0));
}

#[test]
fn rejects_nonfinite_accelerations() {
    assert_eq!(
        acceleration_shares(v(f64::INFINITY, 0.0), v(1.0, 0.0)),
        Err(AllocationError::NonFiniteInput)
    );
}
