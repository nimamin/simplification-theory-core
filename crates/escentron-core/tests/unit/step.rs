use super::*;

fn c() -> ModelConstants {
    ModelConstants {
        g: 1.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: 10.0,
    }
}
fn target(mass: f64, pulse: f64, velocity: Vector3) -> ParticleState {
    ParticleState { position: Vector3::ZERO, velocity, mass, pulse }
}
fn v(x: f64, y: f64) -> Vector3 { Vector3::new(x, y, 0.0) }

#[test]
fn gravitational_acceleration_spends_only_gravitational_energy() {
    let p = propose_step(&target(6.0, 3.0, v(1.0, 0.0)), &c(),
        v(1.0, 0.0), Vector3::ZERO, 1.0).unwrap();
    assert_eq!(p.proposed_velocity, v(2.0, 0.0));
    assert_eq!(p.kinetic_change, 3.0);
    assert_eq!(p.proposed_energies.gravitational, 3.0);
    assert_eq!(p.proposed_energies.levitational, 3.0);
    assert_eq!(p.proposed_energies.kinetic, 4.0);
    assert_eq!(p.proposed_energies.total(), 10.0);
}

#[test]
fn co_directional_channels_share_finite_step() {
    let p = propose_step(&target(4.0, 5.0, v(1.0, 0.0)), &c(),
        v(0.5, 0.0), v(0.5, 0.0), 1.0).unwrap();
    assert_eq!(p.kinetic_change, 3.0);
    assert_eq!(p.gravitational_kinetic_change, 1.5);
    assert_eq!(p.levitational_kinetic_change, 1.5);
    assert_eq!(p.proposed_energies.total(), 10.0);
}

#[test]
fn opposing_weaker_channel_exchanges_no_energy() {
    let p = propose_step(&target(6.0, 3.0, v(1.0, 0.0)), &c(),
        v(2.0, 0.0), v(-1.0, 0.0), 1.0).unwrap();
    assert_eq!(p.kinetic_change, 3.0);
    assert_eq!(p.gravitational_kinetic_change, 3.0);
    assert_eq!(p.levitational_kinetic_change, 0.0);
}

#[test]
fn perpendicular_acceleration_changes_speed_at_second_order() {
    let p = propose_step(&target(6.0, 3.0, v(1.0, 0.0)), &c(),
        v(0.0, 1.0), Vector3::ZERO, 1.0).unwrap();
    assert_eq!(p.proposed_velocity, v(1.0, 1.0));
    assert_eq!(p.kinetic_change, 1.0);
}

#[test]
fn braking_returns_kinetic_energy_to_gravitational_reservoir() {
    let p = propose_step(&target(5.0, 1.0, v(2.0, 0.0)), &c(),
        v(-1.0, 0.0), Vector3::ZERO, 1.0).unwrap();
    assert_eq!(p.kinetic_change, -3.0);
    assert_eq!(p.proposed_energies.gravitational, 8.0);
    assert_eq!(p.proposed_energies.kinetic, 1.0);
    assert_eq!(p.proposed_energies.total(), 10.0);
}

#[test]
fn impossible_energy_request_is_rejected() {
    assert_eq!(
        propose_step(&target(1.0, 8.0, v(1.0, 0.0)), &c(),
            v(2.0, 0.0), Vector3::ZERO, 1.0),
        Err(StepError::Transfer(TransferError::InsufficientSourceEnergy))
    );
}

#[test]
fn zero_net_acceleration_changes_no_energy() {
    let p = propose_step(&target(6.0, 3.0, v(1.0, 0.0)), &c(),
        v(1.0, 0.0), v(-1.0, 0.0), 1.0).unwrap();
    assert_eq!(p.kinetic_change, 0.0);
    assert_eq!(p.proposed_energies.gravitational, 6.0);
    assert_eq!(p.proposed_energies.levitational, 3.0);
}

#[test]
fn rejects_initial_energy_mismatch() {
    assert_eq!(
        propose_step(&target(4.0, 3.0, v(1.0, 0.0)), &c(),
            v(1.0, 0.0), Vector3::ZERO, 1.0),
        Err(StepError::EnergyNotConservedInitially)
    );
}
