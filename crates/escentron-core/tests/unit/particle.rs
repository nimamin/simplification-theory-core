use super::*;

#[test]
fn neutral_radius_is_pulse_over_mass() {
    let state = ParticleState {
        position: Vector3::ZERO, velocity: Vector3::ZERO, mass: 2.0, pulse: 6.0,
    };
    assert_eq!(state.neutral_radius(), Some(3.0));
}

#[test]
fn zero_mass_is_not_an_escentron() {
    let state = ParticleState {
        position: Vector3::ZERO, velocity: Vector3::ZERO, mass: 0.0, pulse: 6.0,
    };
    assert_eq!(state.validate(), Err(ValidationError::NonPositive("mass")));
    assert_eq!(state.neutral_radius(), None);
}

#[test]
fn zero_pulse_is_not_an_escentron() {
    let state = ParticleState {
        position: Vector3::ZERO, velocity: Vector3::ZERO, mass: 1.0, pulse: 0.0,
    };
    assert_eq!(state.validate(), Err(ValidationError::NonPositive("pulse")));
    assert_eq!(state.neutral_radius(), None);
}

#[test]
fn invalid_position_is_rejected() {
    let state = ParticleState {
        position: Vector3::new(f64::NAN, 0.0, 0.0),
        velocity: Vector3::ZERO, mass: 1.0, pulse: 1.0,
    };
    assert_eq!(state.validate(), Err(ValidationError::NotFinite("position")));
}
