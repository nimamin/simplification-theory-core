//! Target-response boundary between independent field types and acceleration.
//!
//! The physical induction laws are not specified in this module. A caller must
//! provide an explicit implementation of each response law. The resulting
//! accelerations, unlike the heterogeneous fields, may be vector-summed.
use crate::{FieldTotals, ParticleState, ValidationError, Vector3};

/// Errors in acceleration contributions or the response calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseError {
    InvalidTarget(ValidationError),
    InvalidField(&'static str),
    InvalidAcceleration(&'static str),
    NonFiniteTotal,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTarget(e) => write!(f, "invalid target: {e}"),
            Self::InvalidField(name) => write!(f, "non-finite {name} field"),
            Self::InvalidAcceleration(name) => write!(f, "non-finite {name} acceleration"),
            Self::NonFiniteTotal => write!(f, "non-finite total acceleration"),
        }
    }
}
impl std::error::Error for ResponseError {}

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
}
