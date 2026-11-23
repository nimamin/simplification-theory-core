//! Algebraic energy bookkeeping. Does not evolve particle state.
use crate::{ModelConstants, ParticleState, ValidationError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyBreakdown {
    pub gravitational: f64,
    pub levitational: f64,
    pub kinetic: f64,
}

impl EnergyBreakdown {
    pub fn total(self) -> f64 {
        self.gravitational + self.levitational + self.kinetic
    }

    /// Difference between computed energy and the model's shared E_c.
    /// This is a diagnostic, not an automatic correction.
    pub fn residual(self, cosmic_energy: f64) -> f64 {
        self.total() - cosmic_energy
    }
}

pub fn energy_of(
    state: &ParticleState,
    constants: &ModelConstants,
) -> Result<EnergyBreakdown, ValidationError> {
    state.validate()?;
    constants.validate()?;
    let energy = EnergyBreakdown {
        gravitational: crate::equations::gravitational_energy(constants.k_g, state.mass),
        levitational: crate::equations::levitational_energy(constants.k_l, state.pulse),
        kinetic: crate::equations::kinetic_energy(constants.k_k, state.velocity),
    };
    if !energy.gravitational.is_finite() { return Err(ValidationError::NotFinite("E_g")); }
    if !energy.levitational.is_finite() { return Err(ValidationError::NotFinite("E_l")); }
    if !energy.kinetic.is_finite() { return Err(ValidationError::NotFinite("E_k")); }
    if !energy.total().is_finite() { return Err(ValidationError::NotFinite("E_total")); }
    Ok(energy)
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/energy.rs"));
}
