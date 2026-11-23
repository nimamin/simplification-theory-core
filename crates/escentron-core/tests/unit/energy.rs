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
