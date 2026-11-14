//! State variables of one Escentron.
use crate::{validation::{finite_positive, ValidationError}, Vector3};

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
        finite_positive(self.mass, "mass")?;
        finite_positive(self.pulse, "pulse")?;
        Ok(())
    }

    /// X = p/m for a valid Escentron. Invalid states return None.
    pub fn neutral_radius(&self) -> Option<f64> {
        if self.validate().is_err() { return None; }
        let radius = self.pulse / self.mass;
        if radius.is_finite() && radius > 0.0 { Some(radius) } else { None }
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
}
