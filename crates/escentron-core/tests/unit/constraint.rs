use super::*;

fn v(x: f64, y: f64, z: f64) -> Vector3 {
    Vector3::new(x, y, z)
}

#[test]
fn fully_depleted_channel_loses_only_forward_component() {
    let result = limit_forward_acceleration(
        v(4.0, 3.0, 2.0), v(1.0, 0.0, 0.0), 0.0,
    ).unwrap();
    assert_eq!(result, v(0.0, 3.0, 2.0));
}

#[test]
fn reverse_component_is_unaffected() {
    let result = limit_forward_acceleration(
        v(-4.0, 3.0, 0.0), v(1.0, 0.0, 0.0), 0.0,
    ).unwrap();
    assert_eq!(result, v(-4.0, 3.0, 0.0));
}

#[test]
fn partial_limit_retains_allowed_forward_acceleration() {
    let result = limit_forward_acceleration(
        v(4.0, 3.0, 0.0), v(1.0, 0.0, 0.0), 1.5,
    ).unwrap();
    assert_eq!(result, v(1.5, 3.0, 0.0));
}

#[test]
fn non_unit_direction_works_without_normalization() {
    let result = limit_forward_acceleration(
        v(4.0, 3.0, 0.0), v(2.0, 0.0, 0.0), 0.0,
    ).unwrap();
    assert_eq!(result, v(0.0, 3.0, 0.0));
}

#[test]
fn inclined_direction_removes_only_parallel_part() {
    let result = limit_forward_acceleration(
        v(3.0, 1.0, 0.0), v(1.0, 1.0, 0.0), 0.0,
    ).unwrap();
    assert_eq!(result, v(1.0, -1.0, 0.0));
}

#[test]
fn zero_direction_is_invalid() {
    assert_eq!(
        limit_forward_acceleration(v(1.0, 0.0, 0.0), Vector3::ZERO, 0.0),
        Err(DirectionalError::ZeroDirection)
    );
}

#[test]
fn nonfinite_input_is_invalid() {
    assert_eq!(
        limit_forward_acceleration(v(f64::NAN, 0.0, 0.0), v(1.0, 0.0, 0.0), 0.0),
        Err(DirectionalError::NonFiniteInput)
    );
}
