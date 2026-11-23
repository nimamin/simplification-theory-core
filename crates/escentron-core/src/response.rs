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
    let coefficient = crate::equations::response_fraction(energy.gravitational, constants.cosmic_energy);
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
    let scaled = crate::equations::apply_response(proposed, alpha);
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
    let coefficient = crate::equations::response_fraction(energy.levitational, constants.cosmic_energy);
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
    let scaled = crate::equations::apply_response(proposed, alpha);
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

/// Confirmed Simplification Theory response: raw field vectors are the
/// unscaled acceleration proposals. The coupling layer subsequently applies
/// E_g/E_c to gravity and E_l/E_c to levitation, separately and once.
///
/// This is an explicit model assumption, not an experimentally validated
/// relation. No additional field-to-acceleration coefficient is introduced.
#[derive(Debug, Clone, Copy, Default)]
pub struct DirectFieldResponse;

impl ResponseLaw for DirectFieldResponse {
    fn from_gravitational(
        &self,
        _target: &ParticleState,
        field: crate::GravitationalField,
    ) -> Vector3 {
        field.0
    }

    fn from_levitational(
        &self,
        _target: &ParticleState,
        field: crate::LevitationalField,
    ) -> Vector3 {
        field.0
    }
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
    let total = crate::equations::resultant(gravitational, levitational);
    if !total.is_finite() { return Err(ResponseError::NonFiniteTotal); }
    Ok(AccelerationBreakdown { gravitational, levitational, total })
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/response.rs"));
}
