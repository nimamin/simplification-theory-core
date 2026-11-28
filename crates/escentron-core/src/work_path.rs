//! Alternative RESEARCH hypothesis: each acceleration channel accounts for its
//! own kinetic work. Not used by the current projection-based step integrator.
//!
//! With constant 3D field vectors, dE_i/dt=-2*k_k*v·a_i and
//! a_i=(E_i/E_c)*f_i imply E_i(x)=E_i0 exp[-2*k_k/E_c*f_i·(x-x0)].
//! This is a path-independent result only for CONSTANT field vectors.
use crate::Vector3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkPathError {
    InvalidInput,
    PrecisionLimit,
    UnreachableKineticEnergy,
}
impl std::fmt::Display for WorkPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for WorkPathError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkPathEnergies {
    pub gravitational: f64,
    pub levitational: f64,
    pub kinetic: f64,
}

/// Hypothetical channel-specific work accounting at a GIVEN displacement.
///
/// Field vectors are fixed throughout the displacement. Does NOT solve
/// position or velocity direction as a function of time. Does NOT alter the
/// canonical energy allocation rule. Never inserts artificial minimum energy.
pub fn constant_field_work_path(
    initial: WorkPathEnergies,
    gravitational_field: Vector3,
    levitational_field: Vector3,
    displacement: Vector3,
    kinetic_coefficient: f64,
    cosmic_energy: f64,
) -> Result<WorkPathEnergies, WorkPathError> {
    let WorkPathEnergies { gravitational: initial_gravitational, levitational: initial_levitational, kinetic: initial_kinetic } = initial;
    let values = [
        initial_gravitational, initial_levitational, initial_kinetic,
        kinetic_coefficient, cosmic_energy,
    ];
    if !values.iter().all(|v| v.is_finite())
        || !gravitational_field.is_finite()
        || !levitational_field.is_finite()
        || !displacement.is_finite()
        || initial_gravitational <= 0.0
        || initial_levitational <= 0.0
        || initial_kinetic < 0.0
        || kinetic_coefficient <= 0.0
        || cosmic_energy <= 0.0
    {
        return Err(WorkPathError::InvalidInput);
    }
    let total = initial_gravitational + initial_levitational + initial_kinetic;
    if !total.is_finite() || (total - cosmic_energy).abs() >
        1e-10 * cosmic_energy.max(1.0) {
        return Err(WorkPathError::InvalidInput);
    }
    let dot = |a: Vector3, b: Vector3| a.x * b.x + a.y * b.y + a.z * b.z;
    let coefficient = -2.0 * kinetic_coefficient / cosmic_energy;
    let eg = initial_gravitational
        * (coefficient * dot(gravitational_field, displacement)).exp();
    let el = initial_levitational
        * (coefficient * dot(levitational_field, displacement)).exp();
    if !eg.is_finite() || !el.is_finite() || eg <= 0.0 || el <= 0.0 {
        return Err(WorkPathError::PrecisionLimit);
    }
    let ek = cosmic_energy - eg - el;
    if !ek.is_finite() {
        return Err(WorkPathError::PrecisionLimit);
    }
    if ek < 0.0 {
        return Err(WorkPathError::UnreachableKineticEnergy);
    }
    Ok(WorkPathEnergies {
        gravitational: eg, levitational: el, kinetic: ek,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noncollinear_field_energies_follow_independent_dot_products() {
        let s = constant_field_work_path(
            WorkPathEnergies { gravitational: 1.0, levitational: 8.0, kinetic: 1.0 },
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::new(0.0, 3.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            1.0, 10.0,
        ).unwrap();
        assert!((s.gravitational - (-0.4_f64).exp()).abs() < 1e-12);
        assert!((s.levitational - 8.0 * (-0.6_f64).exp()).abs() < 1e-12);
        assert!((s.gravitational + s.levitational + s.kinetic - 10.0).abs() < 1e-12);
    }

    #[test]
    fn zero_displacement_preserves_all_energies() {
        let s = constant_field_work_path(
            WorkPathEnergies { gravitational: 1.0, levitational: 8.0, kinetic: 1.0 },
            Vector3::new(3.0, 0.0, 0.0),
            Vector3::new(-2.0, 0.0, 1.0),
            Vector3::ZERO, 1.0, 10.0,
        ).unwrap();
        assert_eq!(s.gravitational, 1.0);
        assert_eq!(s.levitational, 8.0);
        assert_eq!(s.kinetic, 1.0);
    }

    #[test]
    fn inaccessible_reverse_displacement_is_rejected() {
        assert_eq!(
            constant_field_work_path(
                WorkPathEnergies { gravitational: 1.0, levitational: 8.0, kinetic: 1.0 },
                Vector3::new(2.0, 0.0, 0.0),
                Vector3::ZERO,
                Vector3::new(-5.0, 0.0, 0.0),
                1.0, 10.0,
            ),
            Err(WorkPathError::UnreachableKineticEnergy)
        );
    }
}
