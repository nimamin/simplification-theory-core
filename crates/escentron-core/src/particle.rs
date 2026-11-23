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
        let radius = crate::equations::neutral_radius(self.pulse, self.mass);
        if radius.is_finite() && radius > 0.0 { Some(radius) } else { None }
    }
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/particle.rs"));
}
