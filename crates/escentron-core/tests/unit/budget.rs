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
