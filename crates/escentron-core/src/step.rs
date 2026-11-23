//! Evaluate a finite velocity step without committing any state mutation.
//!
//! This is a diagnostic/proposal stage, not the final Escentron integrator.
//! Field coupling, response coefficients and acceleration limits are upstream.
//! No energy transfers between distinct particles or directly between
//! gravitational and levitational reservoirs.
use crate::{
    acceleration_shares, allocate_kinetic_change, energy_of, transfer_energy,
    AllocationError, EnergyBreakdown, EnergyReservoir, ModelConstants,
    ParticleState, TransferError, ValidationError, Vector3,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepError {
    InvalidInput(ValidationError),
    InvalidTimeStep,
    NonFiniteResult,
    EnergyNotConservedInitially,
    Allocation(AllocationError),
    Transfer(TransferError),
    EnergyMismatch,
}

impl From<ValidationError> for StepError {
    fn from(e: ValidationError) -> Self { Self::InvalidInput(e) }
}
impl From<AllocationError> for StepError {
    fn from(e: AllocationError) -> Self { Self::Allocation(e) }
}
impl From<TransferError> for StepError {
    fn from(e: TransferError) -> Self { Self::Transfer(e) }
}
impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for StepError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepProposal {
    pub proposed_velocity: Vector3,
    pub total_acceleration: Vector3,
    pub kinetic_change: f64,
    pub gravitational_kinetic_change: f64,
    pub levitational_kinetic_change: f64,
    pub proposed_energies: EnergyBreakdown,
}

/// Suggest a velocity and energy change over dt > 0.
///
/// Computes v' = v + (a_g + a_l)*dt, then the exact finite-step difference
/// delta_E_k = k_k (|v'|² - |v|²), with no sqrt or normalization.
/// Allocation uses dot products with total acceleration and zeroes canceled
/// contributions. The two assigned energy changes are individually exchanged
/// with kinetic energy only. A step demanding unavailable energy is rejected.
///
/// IMPORTANT: This is an unconstrained candidate, NOT the final time evolution
/// or a guarantee that this acceleration is physically realizable. The full
/// theory may require additional corrections before accepting a candidate.
pub fn propose_step(
    target: &ParticleState,
    constants: &ModelConstants,
    gravitational_acceleration: Vector3,
    levitational_acceleration: Vector3,
    dt: f64,
) -> Result<StepProposal, StepError> {
    if !dt.is_finite() || dt <= 0.0 { return Err(StepError::InvalidTimeStep); }
    let initial = energy_of(target, constants)?;
    if constants.k_k <= 0.0 {
        return Err(StepError::InvalidInput(ValidationError::NonPositive("k_k")));
    }
    let tolerance = 1e-10 * constants.cosmic_energy.max(1.0);
    if (initial.total() - constants.cosmic_energy).abs() > tolerance {
        return Err(StepError::EnergyNotConservedInitially);
    }
    let shares = acceleration_shares(gravitational_acceleration, levitational_acceleration)?;
    let a = shares.total;
    let velocity = crate::equations::advance(target.velocity, a, dt);
    if !velocity.is_finite() { return Err(StepError::NonFiniteResult); }
    let next_kinetic = crate::equations::kinetic_energy(constants.k_k, velocity);
    if !next_kinetic.is_finite() { return Err(StepError::NonFiniteResult); }
    let delta = crate::equations::kinetic_change(next_kinetic, initial.kinetic);
    if !delta.is_finite() { return Err(StepError::NonFiniteResult); }
    let (g_delta, l_delta) = allocate_kinetic_change(shares, delta)?;
    let after_g = transfer_energy(initial, EnergyReservoir::Gravitational, g_delta)?;
    let after_l = transfer_energy(after_g, EnergyReservoir::Levitational, l_delta)?;
    // Compensate only numerical roundoff; actual energy validity is checked
    // rather than silently altering the proposed velocity or source shares.
    if (after_l.kinetic - next_kinetic).abs() > tolerance
        || (after_l.total() - initial.total()).abs() > tolerance
    {
        return Err(StepError::EnergyMismatch);
    }
    Ok(StepProposal {
        proposed_velocity: velocity,
        total_acceleration: a,
        kinetic_change: delta,
        gravitational_kinetic_change: g_delta,
        levitational_kinetic_change: l_delta,
        proposed_energies: after_l,
    })
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/step.rs"));
}
