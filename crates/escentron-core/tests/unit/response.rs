use super::*;
use crate::{GravitationalField, LevitationalField};

// A test double for checking software composition; NOT a physical law.
struct TestResponse;
impl ResponseLaw for TestResponse {
    fn from_gravitational(
        &self, _target: &ParticleState, field: GravitationalField,
    ) -> Vector3 { field.0 }
    fn from_levitational(
        &self, _target: &ParticleState, field: LevitationalField,
    ) -> Vector3 { field.0 }
}

fn target() -> ParticleState {
    ParticleState {
        position: Vector3::ZERO, velocity: Vector3::ZERO, mass: 1.0, pulse: 1.0,
    }
}

#[test]
fn acceleration_components_are_combined_only_after_response() {
    let fields = FieldTotals {
        gravitational: GravitationalField(Vector3::new(-2.0, 1.0, 0.0)),
        levitational: LevitationalField(Vector3::new(3.0, 0.0, 2.0)),
    };
    let result = acceleration_from_fields(&target(), &fields, &TestResponse).unwrap();
    assert_eq!(result.gravitational, Vector3::new(-2.0, 1.0, 0.0));
    assert_eq!(result.levitational, Vector3::new(3.0, 0.0, 2.0));
    assert_eq!(result.total, Vector3::new(1.0, 1.0, 2.0));
}

#[test]
fn non_finite_field_is_rejected_before_response() {
    let fields = FieldTotals {
        gravitational: GravitationalField(Vector3::new(f64::NAN, 0.0, 0.0)),
        levitational: LevitationalField(Vector3::ZERO),
    };
    assert_eq!(
        acceleration_from_fields(&target(), &fields, &TestResponse),
        Err(ResponseError::InvalidField("gravitational"))
    );
}

#[test]
fn gravitational_fraction_is_gravitational_energy_over_cosmic_energy() {
    let mut state = target();
    state.mass = 2.0;
    let constants = crate::ModelConstants {
        g: 1.0, k_g: 10.0, k_l: 0.0, k_k: 0.0, cosmic_energy: 100.0,
    };
    assert_eq!(gravitational_response_coefficient(&state, &constants).unwrap(), 0.2);
    let proposed = Vector3::new(5.0, -10.0, 0.0);
    assert_eq!(
        scale_gravitational_acceleration(&state, &constants, proposed).unwrap(),
        Vector3::new(1.0, -2.0, 0.0)
    );
}

#[test]
fn zero_mass_is_rejected_as_invalid_particle() {
    let mut state = target();
    state.mass = 0.0;
    let constants = crate::ModelConstants {
        g: 1.0, k_g: 10.0, k_l: 0.0, k_k: 0.0, cosmic_energy: 100.0,
    };
    assert_eq!(
        gravitational_response_coefficient(&state, &constants),
        Err(ResponseError::InvalidTarget(ValidationError::NonPositive("mass")))
    );
}

#[test]
fn response_rejects_gravitational_energy_over_budget() {
    let mut state = target();
    state.mass = 11.0;
    let constants = crate::ModelConstants {
        g: 1.0, k_g: 10.0, k_l: 0.0, k_k: 0.0, cosmic_energy: 100.0,
    };
    assert_eq!(
        gravitational_response_coefficient(&state, &constants),
        Err(ResponseError::EnergyExceedsCosmic)
    );
}

#[test]
fn proposed_acceleration_must_be_finite() {
    let constants = crate::ModelConstants {
        g: 1.0, k_g: 10.0, k_l: 0.0, k_k: 0.0, cosmic_energy: 100.0,
    };
    assert_eq!(
        scale_gravitational_acceleration(
            &target(), &constants, Vector3::new(f64::INFINITY, 0.0, 0.0),
        ),
        Err(ResponseError::InvalidAcceleration("proposed gravitational"))
    );
}


#[test]
fn levitational_fraction_is_levitational_energy_over_cosmic_energy() {
    let mut state = target();
    state.pulse = 3.0;
    let constants = crate::ModelConstants {
        g: 1.0, k_g: 0.0, k_l: 10.0, k_k: 0.0, cosmic_energy: 100.0,
    };
    assert_eq!(levitational_response_coefficient(&state, &constants).unwrap(), 0.3);
    assert_eq!(
        scale_levitational_acceleration(
            &state, &constants, Vector3::new(10.0, -20.0, 0.0),
        ).unwrap(),
        Vector3::new(3.0, -6.0, 0.0)
    );
}

#[test]
fn zero_pulse_is_rejected_as_invalid_particle() {
    let mut state = target();
    state.pulse = 0.0;
    let constants = crate::ModelConstants {
        g: 1.0, k_g: 0.0, k_l: 10.0, k_k: 0.0, cosmic_energy: 100.0,
    };
    assert_eq!(
        levitational_response_coefficient(&state, &constants),
        Err(ResponseError::InvalidTarget(ValidationError::NonPositive("pulse")))
    );
}

#[test]
fn levitational_energy_over_budget_is_rejected() {
    let mut state = target();
    state.pulse = 11.0;
    let constants = crate::ModelConstants {
        g: 1.0, k_g: 0.0, k_l: 10.0, k_k: 0.0, cosmic_energy: 100.0,
    };
    assert_eq!(
        levitational_response_coefficient(&state, &constants),
        Err(ResponseError::EnergyExceedsCosmic)
    );
}

#[test]
fn nonfinite_proposed_levitational_acceleration_is_rejected() {
    let constants = crate::ModelConstants {
        g: 1.0, k_g: 0.0, k_l: 10.0, k_k: 0.0, cosmic_energy: 100.0,
    };
    assert_eq!(
        scale_levitational_acceleration(
            &target(), &constants, Vector3::new(f64::NAN, 0.0, 0.0),
        ),
        Err(ResponseError::InvalidAcceleration("proposed levitational"))
    );
}
