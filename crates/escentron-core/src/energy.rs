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
        gravitational: constants.k_g * state.mass,
        levitational: constants.k_l * state.pulse,
        kinetic: constants.k_k * state.velocity.magnitude_squared(),
    };
    if !energy.gravitational.is_finite() { return Err(ValidationError::NotFinite("E_g")); }
    if !energy.levitational.is_finite() { return Err(ValidationError::NotFinite("E_l")); }
    if !energy.kinetic.is_finite() { return Err(ValidationError::NotFinite("E_k")); }
    if !energy.total().is_finite() { return Err(ValidationError::NotFinite("E_total")); }
    Ok(energy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector3;

    #[test]
    fn computes_three_energy_channels() {
        let state = ParticleState {
            position: Vector3::ZERO, velocity: Vector3::new(3.0, 4.0, 0.0),
            mass: 2.0, pulse: 3.0,
        };
        let constants = ModelConstants {
            g: 1.0, k_g: 2.0, k_l: 3.0, k_k: 4.0, cosmic_energy: 113.0,
        };
        let e = energy_of(&state, &constants).unwrap();
        assert_eq!(e.gravitational, 4.0);
        assert_eq!(e.levitational, 9.0);
        assert_eq!(e.kinetic, 100.0);
        assert_eq!(e.total(), 113.0);
        assert_eq!(e.residual(constants.cosmic_energy), 0.0);
    }

    #[test]
    fn energy_mismatch_is_reported_not_hidden() {
        let e = EnergyBreakdown { gravitational: 2.0, levitational: 3.0, kinetic: 4.0 };
        assert_eq!(e.residual(10.0), -1.0);
    }
}
