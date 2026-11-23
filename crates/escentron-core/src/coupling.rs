//! Explicit adapter: external source fields -> caller-defined raw acceleration
//! proposals -> energy-fraction scaling -> reference simulation.
//!
//! Only the target evolves. All other source particles are frozen during the
//! reference interval. This is NOT a self-consistent multi-body simulation.
use crate::{
    acceleration_from_fields, fields_for_target_state,
    gravitational_response_coefficient, levitational_response_coefficient,
    AccelerationBreakdown,
    ModelConstants, ParticleState, ResponseError, ResponseLaw,
    Vector3, FieldTotals, DirectFieldResponse,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CouplingError {
    Field(crate::FieldError),
    Response(ResponseError),
}

impl From<crate::FieldError> for CouplingError {
    fn from(error: crate::FieldError) -> Self { Self::Field(error) }
}
impl From<ResponseError> for CouplingError {
    fn from(error: ResponseError) -> Self { Self::Response(error) }
}

/// Full auditable breakdown at a single position. The two field types
/// remain separate and are never numerically added.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CouplingTrace {
    pub fields: FieldTotals,
    /// Raw acceleration proposals from a supplied, explicitly chosen law.
    pub proposed: AccelerationBreakdown,
    pub gravitational_energy_fraction: f64,
    pub levitational_energy_fraction: f64,
    /// The two response-scaled accelerations and their vector resultant.
    pub applied: AccelerationBreakdown,
}

fn scale_vector(vector: Vector3, factor: f64) -> Vector3 {
    crate::equations::apply_response(vector, factor)
}

fn checked_acceleration(
    gravitational: Vector3,
    levitational: Vector3,
) -> Result<AccelerationBreakdown, CouplingError> {
    if !gravitational.is_finite() {
        return Err(CouplingError::Response(ResponseError::InvalidAcceleration("gravitational")));
    }
    if !levitational.is_finite() {
        return Err(CouplingError::Response(ResponseError::InvalidAcceleration("levitational")));
    }
    let total = crate::equations::resultant(gravitational, levitational);
    if !total.is_finite() {
        return Err(CouplingError::Response(ResponseError::NonFiniteTotal));
    }
    Ok(AccelerationBreakdown { gravitational, levitational, total })
}

/// Trace the complete *known* part of field-to-acceleration processing:
/// separate fields -> explicit raw law -> individual energy fractions
/// -> separately scaled acceleration vectors.
///
/// This function intentionally does not assume any missing physical
/// relation for conversion from fields to raw acceleration proposals.
pub fn coupling_trace(
    sources: &[ParticleState],
    target_index: usize,
    current_target: &ParticleState,
    constants: &ModelConstants,
    law: &impl ResponseLaw,
) -> Result<CouplingTrace, CouplingError> {
    let fields = fields_for_target_state(sources, target_index, current_target, constants)?;
    let proposed = acceleration_from_fields(current_target, &fields, law)?;
    let gravitational_energy_fraction =
        gravitational_response_coefficient(current_target, constants)?;
    let levitational_energy_fraction =
        levitational_response_coefficient(current_target, constants)?;
    let applied = checked_acceleration(
        scale_vector(proposed.gravitational, gravitational_energy_fraction),
        scale_vector(proposed.levitational, levitational_energy_fraction),
    )?;
    Ok(CouplingTrace {
        fields,
        proposed,
        gravitational_energy_fraction,
        levitational_energy_fraction,
        applied,
    })
}

/// Lightweight path for the reference solver. The independent field response
/// and the two energy fractions are applied exactly once.
pub fn coupled_acceleration(
    sources: &[ParticleState],
    target_index: usize,
    current_target: &ParticleState,
    constants: &ModelConstants,
    law: &impl ResponseLaw,
) -> Result<AccelerationBreakdown, CouplingError> {
    Ok(coupling_trace(sources, target_index, current_target, constants, law)?.applied)
}

/// The confirmed model acceleration for the target under other sources:
/// a_g = (E_g/E_c) * f_g; a_l = (E_l/E_c) * f_l.
/// The two acceleration vectors, unlike their source field types, can sum.
pub fn model_acceleration(
    sources: &[ParticleState],
    target_index: usize,
    current_target: &ParticleState,
    constants: &ModelConstants,
) -> Result<AccelerationBreakdown, CouplingError> {
    coupled_acceleration(
        sources, target_index, current_target, constants, &DirectFieldResponse,
    )
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/coupling.rs"));
}
