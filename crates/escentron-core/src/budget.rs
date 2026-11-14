//! Energy-budget diagnostics for the proposed Escentron model.
//!
//! These bounds follow algebraically from E_c = E_g + E_l + k_k |v|².
//! This module never changes particle state, clips velocities, or decides
//! which reservoir can give or receive energy during acceleration.
use crate::{energy_of, ModelConstants, ParticleState, ValidationError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetError {
    InvalidInput(ValidationError),
    ZeroKineticCoefficient,
    ReservoirsExceedCosmicEnergy,
    KineticEnergyExceedsBudget,
    NonFiniteResult,
}

impl From<ValidationError> for BudgetError {
    fn from(value: ValidationError) -> Self { Self::InvalidInput(value) }
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(err) => write!(f, "{err}"),
            Self::ZeroKineticCoefficient => write!(f, "k_k must be positive to infer a speed bound"),
            Self::ReservoirsExceedCosmicEnergy => write!(f, "E_g + E_l exceeds E_c"),
            Self::KineticEnergyExceedsBudget => write!(f, "current kinetic energy exceeds available energy"),
            Self::NonFiniteResult => write!(f, "kinetic budget calculation produced a non-finite value"),
        }
    }
}

impl std::error::Error for BudgetError {}

/// Read-only budget for the target Escentron at the current energy split.
/// `max_speed_with_current_reservoirs` assumes E_g and E_l are held fixed.
/// It is NOT a universal speed limit if energy can transfer from reservoirs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KineticBudget {
    pub gravitational_energy: f64,
    pub levitational_energy: f64,
    pub kinetic_energy: f64,
    pub available_kinetic_energy: f64,
    pub remaining_kinetic_capacity: f64,
    pub max_speed_with_current_reservoirs: f64,
    /// Upper bound when both non-kinetic reservoirs could be fully depleted.
    /// Whether such conversion is dynamically possible is not assumed.
    pub absolute_energy_speed_bound: f64,
}

pub fn kinetic_budget(
    target: &ParticleState,
    constants: &ModelConstants,
) -> Result<KineticBudget, BudgetError> {
    let energy = energy_of(target, constants)?;
    if constants.k_k == 0.0 {
        return Err(BudgetError::ZeroKineticCoefficient);
    }

    let reservoir_total = energy.gravitational + energy.levitational;
    if !reservoir_total.is_finite() {
        return Err(BudgetError::NonFiniteResult);
    }
    if reservoir_total > constants.cosmic_energy {
        return Err(BudgetError::ReservoirsExceedCosmicEnergy);
    }

    let available = constants.cosmic_energy - reservoir_total;
    if energy.kinetic > available {
        return Err(BudgetError::KineticEnergyExceedsBudget);
    }
    let remaining = available - energy.kinetic;
    let max_speed = (available / constants.k_k).sqrt();
    let absolute_bound = (constants.cosmic_energy / constants.k_k).sqrt();
    if !available.is_finite()
        || !remaining.is_finite()
        || !max_speed.is_finite()
        || !absolute_bound.is_finite()
    {
        return Err(BudgetError::NonFiniteResult);
    }

    Ok(KineticBudget {
        gravitational_energy: energy.gravitational,
        levitational_energy: energy.levitational,
        kinetic_energy: energy.kinetic,
        available_kinetic_energy: available,
        remaining_kinetic_capacity: remaining,
        max_speed_with_current_reservoirs: max_speed,
        absolute_energy_speed_bound: absolute_bound,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vector3;

    fn constants() -> ModelConstants {
        ModelConstants {
            g: 1.0, k_g: 2.0, k_l: 3.0, k_k: 4.0, cosmic_energy: 113.0,
        }
    }

    fn target() -> ParticleState {
        ParticleState {
            position: Vector3::ZERO,
            velocity: Vector3::new(3.0, 4.0, 0.0),
            mass: 2.0,
            pulse: 3.0,
        }
    }

    #[test]
    fn exact_energy_split_gives_zero_headroom() {
        let b = kinetic_budget(&target(), &constants()).unwrap();
        assert_eq!(b.gravitational_energy, 4.0);
        assert_eq!(b.levitational_energy, 9.0);
        assert_eq!(b.kinetic_energy, 100.0);
        assert_eq!(b.available_kinetic_energy, 100.0);
        assert_eq!(b.remaining_kinetic_capacity, 0.0);
        assert_eq!(b.max_speed_with_current_reservoirs, 5.0);
    }

    #[test]
    fn at_rest_budget_is_entirely_available_for_future_kinetic_energy() {
        let mut t = target();
        t.velocity = Vector3::ZERO;
        let b = kinetic_budget(&t, &constants()).unwrap();
        assert_eq!(b.remaining_kinetic_capacity, 100.0);
        assert_eq!(b.max_speed_with_current_reservoirs, 5.0);
    }

    #[test]
    fn kinetic_overspend_is_rejected() {
        let mut t = target();
        t.velocity = Vector3::new(6.0, 0.0, 0.0);
        assert_eq!(
            kinetic_budget(&t, &constants()),
            Err(BudgetError::KineticEnergyExceedsBudget)
        );
    }

    #[test]
    fn nonkinetic_overspend_is_rejected() {
        let mut t = target();
        t.mass = 100.0;
        assert_eq!(
            kinetic_budget(&t, &constants()),
            Err(BudgetError::ReservoirsExceedCosmicEnergy)
        );
    }

    #[test]
    fn zero_kinetic_conversion_is_undefined_for_speed_bound() {
        let mut c = constants();
        c.k_k = 0.0;
        assert_eq!(
            kinetic_budget(&target(), &c),
            Err(BudgetError::ZeroKineticCoefficient)
        );
    }

    #[test]
    fn no_reservoirs_give_absolute_energy_speed_bound() {
        let mut t = target();
        t.mass = 0.0;
        t.pulse = 0.0;
        t.velocity = Vector3::ZERO;
        let b = kinetic_budget(&t, &constants()).unwrap();
        assert_eq!(b.absolute_energy_speed_bound, (113.0_f64 / 4.0).sqrt());
        assert_eq!(b.max_speed_with_current_reservoirs, b.absolute_energy_speed_bound);
    }
}
