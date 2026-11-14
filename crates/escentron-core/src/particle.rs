//! State variables of one Escentron.
use crate::{validation::{finite_nonnegative, ValidationError}, Vector3};

/// Instantaneous state. No acceleration or integration step is implied.
/// 'mass' (m) and 'pulse' (p) are the model's named physical parameters.
/// Their physical units and calibration remain to be established.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleState {
    pub position: Vector3,
    pub velocity: Vector3,
    pub mass: f64,
    pub pulse: f64,
}

impl ParticleState {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !self.position.is_finite() { return Err(ValidationError::NotFinite("position")); }
        if !self.velocity.is_finite() { return Err(ValidationError::NotFinite("velocity")); }
        finite_nonnegative(self.mass, "mass")?;
        finite_nonnegative(self.pulse, "pulse")?;
        Ok(())
    }

    /// X = p/m, defined only when m > 0.
    ///
    /// Does not assign a special physical interpretation to m=0.
    pub fn neutral_radius(&self) -> Option<f64> {
        if self.mass > 0.0 { Some(self.pulse / self.mass) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_radius_is_pulse_over_mass() {
        let state = ParticleState {
            position: Vector3::ZERO, velocity: Vector3::ZERO, mass: 2.0, pulse: 6.0,
        };
        assert_eq!(state.neutral_radius(), Some(3.0));
    }

    #[test]
    fn zero_mass_has_no_computed_neutral_radius() {
        let state = ParticleState {
            position: Vector3::ZERO, velocity: Vector3::ZERO, mass: 0.0, pulse: 6.0,
        };
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
}
