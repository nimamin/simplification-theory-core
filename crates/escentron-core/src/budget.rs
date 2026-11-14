//! Accounting for energy exchange between one source reservoir and kinetic energy.
//!
//! These functions do not determine the amount of energy to exchange.
//! Field response, acceleration, timestep and velocity update are separate.
use crate::EnergyBreakdown;

/// Which of the Escentron's own reservoirs exchanges energy with kinetic energy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyReservoir {
    Gravitational,
    Levitational,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferError {
    NonFinite,
    NegativeEnergy,
    NonPositiveFieldEnergy,
    InsufficientSourceEnergy,
    InsufficientKineticEnergy,
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFinite => write!(f, "energy transfer requires finite values"),
            Self::NegativeEnergy => write!(f, "energy reservoirs cannot be negative"),
            Self::NonPositiveFieldEnergy => write!(f, "gravitational and levitational reservoirs must remain positive"),
            Self::InsufficientSourceEnergy =>
                write!(f, "selected reservoir cannot supply the requested kinetic increase"),
            Self::InsufficientKineticEnergy =>
                write!(f, "kinetic reservoir cannot supply the requested reverse transfer"),
        }
    }
}
impl std::error::Error for TransferError {}

/// Transfer a *specified* signed kinetic-energy change between one selected
/// reservoir and the kinetic reservoir of the SAME Escentron.
///
/// delta_kinetic > 0: selected reservoir -> kinetic energy
/// delta_kinetic < 0: kinetic energy -> selected reservoir
///
/// The other non-kinetic reservoir is unchanged. Energy is not transferred
/// between Escentrons. This is bookkeeping, not an acceleration algorithm.
/// The caller must determine delta_kinetic from the final dynamics law.
pub fn transfer_energy(
    current: EnergyBreakdown,
    source: EnergyReservoir,
    delta_kinetic: f64,
) -> Result<EnergyBreakdown, TransferError> {
    let values = [
        current.gravitational,
        current.levitational,
        current.kinetic,
        delta_kinetic,
    ];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(TransferError::NonFinite);
    }
    if current.gravitational < 0.0
        || current.levitational < 0.0
        || current.kinetic < 0.0
    {
        return Err(TransferError::NegativeEnergy);
    }

    if current.gravitational == 0.0 || current.levitational == 0.0 {
        return Err(TransferError::NonPositiveFieldEnergy);
    }

    let source_energy = match source {
        EnergyReservoir::Gravitational => current.gravitational,
        EnergyReservoir::Levitational => current.levitational,
    };

    if delta_kinetic > source_energy {
        return Err(TransferError::InsufficientSourceEnergy);
    }
    if delta_kinetic < -current.kinetic {
        return Err(TransferError::InsufficientKineticEnergy);
    }

    let updated_source = source_energy - delta_kinetic;
    let updated_kinetic = current.kinetic + delta_kinetic;
    if !updated_source.is_finite() || !updated_kinetic.is_finite() {
        return Err(TransferError::NonFinite);
    }

    if updated_source == 0.0 {
        return Err(TransferError::NonPositiveFieldEnergy);
    }

    let mut updated = current;
    match source {
        EnergyReservoir::Gravitational => updated.gravitational = updated_source,
        EnergyReservoir::Levitational => updated.levitational = updated_source,
    }
    updated.kinetic = updated_kinetic;
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initial() -> EnergyBreakdown {
        EnergyBreakdown { gravitational: 40.0, levitational: 30.0, kinetic: 30.0 }
    }

    #[test]
    fn gravitational_field_transfers_gravitational_to_kinetic() {
        let before = initial();
        let after = transfer_energy(before, EnergyReservoir::Gravitational, 12.0).unwrap();
        assert_eq!(after.gravitational, 28.0);
        assert_eq!(after.levitational, 30.0);
        assert_eq!(after.kinetic, 42.0);
        assert_eq!(after.total(), before.total());
    }

    #[test]
    fn levitational_field_transfers_levitational_to_kinetic() {
        let before = initial();
        let after = transfer_energy(before, EnergyReservoir::Levitational, 10.0).unwrap();
        assert_eq!(after.gravitational, 40.0);
        assert_eq!(after.levitational, 20.0);
        assert_eq!(after.kinetic, 40.0);
        assert_eq!(after.total(), before.total());
    }

    #[test]
    fn reverse_gravitational_exchange_returns_kinetic_to_gravitational() {
        let before = initial();
        let after = transfer_energy(before, EnergyReservoir::Gravitational, -10.0).unwrap();
        assert_eq!(after.gravitational, 50.0);
        assert_eq!(after.levitational, 30.0);
        assert_eq!(after.kinetic, 20.0);
        assert_eq!(after.total(), before.total());
    }

    #[test]
    fn reverse_levitational_exchange_returns_kinetic_to_levitational() {
        let before = initial();
        let after = transfer_energy(before, EnergyReservoir::Levitational, -10.0).unwrap();
        assert_eq!(after.gravitational, 40.0);
        assert_eq!(after.levitational, 40.0);
        assert_eq!(after.kinetic, 20.0);
        assert_eq!(after.total(), before.total());
    }

    #[test]
    fn rejects_source_overspend() {
        assert_eq!(
            transfer_energy(initial(), EnergyReservoir::Levitational, 31.0),
            Err(TransferError::InsufficientSourceEnergy)
        );
    }

    #[test]
    fn rejects_kinetic_overspend() {
        assert_eq!(
            transfer_energy(initial(), EnergyReservoir::Gravitational, -31.0),
            Err(TransferError::InsufficientKineticEnergy)
        );
    }

    #[test]
    fn cannot_deplete_gravitational_energy_to_zero() {
        assert_eq!(
            transfer_energy(initial(), EnergyReservoir::Gravitational, 40.0),
            Err(TransferError::NonPositiveFieldEnergy)
        );
    }

    #[test]
    fn cannot_deplete_levitational_energy_to_zero() {
        assert_eq!(
            transfer_energy(initial(), EnergyReservoir::Levitational, 30.0),
            Err(TransferError::NonPositiveFieldEnergy)
        );
    }

    #[test]
    fn rejects_nonfinite_transfer() {
        assert_eq!(
            transfer_energy(initial(), EnergyReservoir::Gravitational, f64::NAN),
            Err(TransferError::NonFinite)
        );
    }
}
