//! Target-response boundary between independent field types and acceleration.
//!
//! The physical induction laws are not specified in this module. A caller must
//! provide an explicit implementation of each response law. The resulting
//! accelerations, unlike the heterogeneous fields, may be vector-summed.
use crate::{energy_of, FieldTotals, ModelConstants, ParticleState, ValidationError, Vector3};

/// Errors in acceleration contributions or the response calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseError {
    InvalidTarget(ValidationError),
    InvalidField(&'static str),
    InvalidAcceleration(&'static str),
    NonFiniteTotal,
    EnergyExceedsCosmic,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTarget(e) => write!(f, "invalid target: {e}"),
            Self::InvalidField(name) => write!(f, "non-finite {name} field"),
            Self::InvalidAcceleration(name) => write!(f, "non-finite {name} acceleration"),
            Self::NonFiniteTotal => write!(f, "non-finite total acceleration"),
            Self::EnergyExceedsCosmic => write!(f, "field-specific energy exceeds cosmic energy"),
        }
    }
}
impl std::error::Error for ResponseError {}


/// Fraction of a target Escentron's cosmic energy stored as gravitational energy.
///
/// alpha_g = E_g / E_c = (k_g * m) / E_c.
///
/// This is one *partial* response coefficient, not a complete acceleration
/// law. The input state must respect the energy budget: E_g <= E_c.
/// No clamping or normalization is performed.
pub fn gravitational_response_coefficient(
    target: &ParticleState,
    constants: &ModelConstants,
) -> Result<f64, ResponseError> {
    let energy = energy_of(target, constants).map_err(ResponseError::InvalidTarget)?;
    if energy.gravitational > constants.cosmic_energy {
        return Err(ResponseError::EnergyExceedsCosmic);
    }
    let coefficient = energy.gravitational / constants.cosmic_energy;
    if !coefficient.is_finite() {
        return Err(ResponseError::InvalidAcceleration("gravitational coefficient"));
    }
    Ok(coefficient)
}

/// Scale a separately determined *proposed* gravitational acceleration
/// by the target's gravitational energy fraction.
///
/// This is NOT a conversion from gravitational field to acceleration.
/// It does not implement the direction-dependent energy transfer, velocity
/// constraint, timestep, or remaining response rules.
pub fn scale_gravitational_acceleration(
    target: &ParticleState,
    constants: &ModelConstants,
    proposed: Vector3,
) -> Result<Vector3, ResponseError> {
    if !proposed.is_finite() {
        return Err(ResponseError::InvalidAcceleration("proposed gravitational"));
    }
    let alpha = gravitational_response_coefficient(target, constants)?;
    let scaled = Vector3::new(
        proposed.x * alpha,
        proposed.y * alpha,
        proposed.z * alpha,
    );
    if !scaled.is_finite() {
        return Err(ResponseError::InvalidAcceleration("scaled gravitational"));
    }
    Ok(scaled)
}


/// Fraction of a target Escentron's cosmic energy stored as levitational energy.
///
/// alpha_l = E_l / E_c = (k_l * p) / E_c.
/// This is a partial response coefficient, not a complete acceleration law.
/// No clamping or energy redistribution is performed.
pub fn levitational_response_coefficient(
    target: &ParticleState,
    constants: &ModelConstants,
) -> Result<f64, ResponseError> {
    let energy = energy_of(target, constants).map_err(ResponseError::InvalidTarget)?;
    if energy.levitational > constants.cosmic_energy {
        return Err(ResponseError::EnergyExceedsCosmic);
    }
    let coefficient = energy.levitational / constants.cosmic_energy;
    if !coefficient.is_finite() {
        return Err(ResponseError::InvalidAcceleration("levitational coefficient"));
    }
    Ok(coefficient)
}

/// Apply the target's levitational energy fraction to an independently
/// determined *proposed* levitational acceleration.
/// This does not convert a field to acceleration or impose a speed limit.
pub fn scale_levitational_acceleration(
    target: &ParticleState,
    constants: &ModelConstants,
    proposed: Vector3,
) -> Result<Vector3, ResponseError> {
    if !proposed.is_finite() {
        return Err(ResponseError::InvalidAcceleration("proposed levitational"));
    }
    let alpha = levitational_response_coefficient(target, constants)?;
    let scaled = Vector3::new(
        proposed.x * alpha,
        proposed.y * alpha,
        proposed.z * alpha,
    );
    if !scaled.is_finite() {
        return Err(ResponseError::InvalidAcceleration("scaled levitational"));
    }
    Ok(scaled)
}

/// Placeholder contract for target-specific physical induction rules.
/// Implementations will be supplied only after their equations are agreed.
/// Receiving two distinct field types prevents implicit field addition.
pub trait ResponseLaw {
    fn from_gravitational(
        &self,
        target: &ParticleState,
        field: crate::GravitationalField,
    ) -> Vector3;

    fn from_levitational(
        &self,
        target: &ParticleState,
        field: crate::LevitationalField,
    ) -> Vector3;
}

/// Separate acceleration contributions, plus their vector sum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccelerationBreakdown {
    pub gravitational: Vector3,
    pub levitational: Vector3,
    pub total: Vector3,
}

/// Apply a specified target-response law to previously computed field totals.
///
/// The law is deliberately external. This function does not assume a coupling
/// coefficient, particle-to-particle transfer, timestep, or force formula.
pub fn acceleration_from_fields(
    target: &ParticleState,
    fields: &FieldTotals,
    law: &impl ResponseLaw,
) -> Result<AccelerationBreakdown, ResponseError> {
    target.validate().map_err(ResponseError::InvalidTarget)?;
    if !fields.gravitational.0.is_finite() {
        return Err(ResponseError::InvalidField("gravitational"));
    }
    if !fields.levitational.0.is_finite() {
        return Err(ResponseError::InvalidField("levitational"));
    }
    let gravitational = law.from_gravitational(target, fields.gravitational);
    let levitational = law.from_levitational(target, fields.levitational);
    if !gravitational.is_finite() {
        return Err(ResponseError::InvalidAcceleration("gravitational"));
    }
    if !levitational.is_finite() {
        return Err(ResponseError::InvalidAcceleration("levitational"));
    }
    let total = Vector3::new(
        gravitational.x + levitational.x,
        gravitational.y + levitational.y,
        gravitational.z + levitational.z,
    );
    if !total.is_finite() { return Err(ResponseError::NonFiniteTotal); }
    Ok(AccelerationBreakdown { gravitational, levitational, total })
}

#[cfg(test)]
mod tests {
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
    fn zero_mass_has_zero_gravitational_response() {
        let mut state = target();
        state.mass = 0.0;
        let constants = crate::ModelConstants {
            g: 1.0, k_g: 10.0, k_l: 0.0, k_k: 0.0, cosmic_energy: 100.0,
        };
        assert_eq!(gravitational_response_coefficient(&state, &constants).unwrap(), 0.0);
        assert_eq!(
            scale_gravitational_acceleration(
                &state, &constants, Vector3::new(100.0, 0.0, 0.0),
            ).unwrap(),
            Vector3::ZERO
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
    fn zero_pulse_has_zero_levitational_response() {
        let mut state = target();
        state.pulse = 0.0;
        let constants = crate::ModelConstants {
            g: 1.0, k_g: 0.0, k_l: 10.0, k_k: 0.0, cosmic_energy: 100.0,
        };
        assert_eq!(levitational_response_coefficient(&state, &constants).unwrap(), 0.0);
        assert_eq!(
            scale_levitational_acceleration(
                &state, &constants, Vector3::new(50.0, 0.0, 0.0),
            ).unwrap(),
            Vector3::ZERO
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

}
