use crate::*;
use super::*;
use crate::{GravitationalField, LevitationalField};

// Software-only mock law: returns vectors unchanged. It is NOT
// the missing physics mapping from field strength to acceleration.
struct IdentityMock;
impl ResponseLaw for IdentityMock {
    fn from_gravitational(&self, _: &ParticleState, f: GravitationalField) -> Vector3 { f.0 }
    fn from_levitational(&self, _: &ParticleState, f: LevitationalField) -> Vector3 { f.0 }
}
struct ZeroMock;
impl ResponseLaw for ZeroMock {
    fn from_gravitational(&self, _: &ParticleState, _: GravitationalField) -> Vector3 { Vector3::ZERO }
    fn from_levitational(&self, _: &ParticleState, _: LevitationalField) -> Vector3 { Vector3::ZERO }
}
fn constants() -> ModelConstants {
    ModelConstants { g: 1.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: 10.0 }
}
fn sources() -> [ParticleState; 2] {
    [
        ParticleState {
            position: Vector3::ZERO,
            velocity: Vector3::new(1.0, 0.0, 0.0),
            mass: 6.0, pulse: 3.0,
        },
        ParticleState {
            position: Vector3::new(10.0, 0.0, 0.0),
            velocity: Vector3::ZERO,
            mass: 2.0, pulse: 6.0,
        },
    ]
}
#[test]
fn applies_each_energy_fraction_once_to_its_own_acceleration() {
    let s = sources();
    let a = coupled_acceleration(&s, 0, &s[0], &constants(), &IdentityMock).unwrap();
    // At x=0 the distant source at x=10 pulls right and pushes left.
    assert!((a.gravitational.x - 0.012).abs() < 1e-12); // (2/100)*(6/10)
    assert!((a.levitational.x + 0.0018).abs() < 1e-12); // (6/1000)*(3/10)
    assert!((a.total.x - 0.0102).abs() < 1e-12);
}
#[test]
fn zero_mock_keeps_both_reservoirs_and_velocity_unchanged() {
    let s = sources();
    let result = reference_with_fixed_sources(
        &s, 0, &constants(),
        RetrySettings { duration: 1.0, initial_steps: 2, max_steps: 32 },
        &ZeroMock,
    ).unwrap();
    assert_eq!(result.state.mass, s[0].mass);
    assert_eq!(result.state.pulse, s[0].pulse);
    assert_eq!(result.state.velocity, s[0].velocity);
    assert_eq!(result.state.position.x, 1.0);
}
#[test]
fn convergence_adapter_produces_agreeing_zero_mock_trajectories() {
    let s = sources();
    let result = convergence_with_fixed_sources(
        &s, 0, &constants(),
        ConvergenceSettings {
            duration: 1.0, initial_steps: 2, max_steps: 32,
            absolute_tolerance: 1e-9, relative_tolerance: 1e-9,
        },
        &ZeroMock,
    ).unwrap();
    assert_eq!(result.reference.accepted_steps, 8);
}
#[test]
fn nonexistent_index_reports_field_error() {
    let s = sources();
    assert_eq!(
        coupled_acceleration(&s, 2, &s[0], &constants(), &ZeroMock),
        Err(CouplingError::Field(crate::FieldError::InvalidTargetIndex))
    );
}
#[test]
fn trace_keeps_raw_fields_proposals_and_energy_fractions_separate() {
    let s = sources();
    let trace = coupling_trace(&s, 0, &s[0], &constants(), &IdentityMock).unwrap();
    assert!((trace.fields.gravitational.0.x - 0.02).abs() < 1e-12);
    assert!((trace.fields.levitational.0.x + 0.006).abs() < 1e-12);
    assert_eq!(trace.gravitational_energy_fraction, 0.6);
    assert_eq!(trace.levitational_energy_fraction, 0.3);
    assert_eq!(trace.proposed.gravitational, trace.fields.gravitational.0);
    assert_eq!(trace.proposed.levitational, trace.fields.levitational.0);
    assert!((trace.applied.gravitational.x - 0.012).abs() < 1e-12);
    assert!((trace.applied.levitational.x + 0.0018).abs() < 1e-12);
}

#[test]
fn moved_target_trace_uses_new_position_not_original() {
    let s = sources();
    let first = coupling_trace(&s, 0, &s[0], &constants(), &IdentityMock).unwrap();
    let mut moved = s[0];
    moved.position = Vector3::new(1.0, 0.0, 0.0);
    let second = coupling_trace(&s, 0, &moved, &constants(), &IdentityMock).unwrap();
    assert_ne!(first.fields.gravitational.0.x, second.fields.gravitational.0.x);
}

#[test]
fn confirmed_model_response_matches_energy_scaled_fields() {
    let s = sources();
    let a = model_acceleration(&s, 0, &s[0], &constants()).unwrap();
    assert!((a.gravitational.x - 0.012).abs() < 1e-12);
    assert!((a.levitational.x + 0.0018).abs() < 1e-12);
    assert!((a.total.x - 0.0102).abs() < 1e-12);
}

#[test]
fn model_reference_runs_with_nonzero_field_and_preserves_positive_energy() {
    let s = sources();
    let result = reference_model_with_fixed_sources(
        &s, 0, &constants(),
        RetrySettings { duration: 1.0, initial_steps: 16, max_steps: 4096 },
    ).unwrap();
    assert!(result.state.velocity.x > s[0].velocity.x);
    assert!(result.state.mass > 0.0);
    assert!(result.state.pulse > 0.0);
    let total = crate::energy_of(&result.state, &constants()).unwrap().total();
    assert!((total - 10.0).abs() < 1e-9);
}

#[test]
fn model_convergence_runs_with_actual_response_law() {
    let s = sources();
    let result = convergence_model_with_fixed_sources(
        &s, 0, &constants(),
        ConvergenceSettings {
            duration: 1.0, initial_steps: 8, max_steps: 4096,
            absolute_tolerance: 1e-5, relative_tolerance: 1e-5,
        },
    ).unwrap();
    assert!(result.reference.accepted_steps >= 32);
    assert!(result.reference.state.mass > 0.0);
    assert!(result.reference.state.pulse > 0.0);
}

// Fixed nucleus is an intentionally simplified single-moving-target test,
// not a claim of a self-consistent electronic atom or quantum orbital.
fn radial_atom_inputs(radius: f64) -> ([ParticleState; 2], ModelConstants) {
    let constants = ModelConstants {
        g: 8.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: 10.0,
    };
    let electron = ParticleState {
        position: Vector3::new(radius, 0.0, 0.0),
        velocity: Vector3::ZERO, mass: 4.0, pulse: 6.0,
    };
    let nucleus = ParticleState {
        position: Vector3::ZERO,
        velocity: Vector3::new(1.0, 0.0, 0.0),
        mass: 6.0, pulse: 3.0,
    };
    ([electron, nucleus], constants)
}

#[test]
fn exactly_balanced_radial_target_keeps_energies_unchanged() {
    // r_eq = (target pulse * nucleus pulse) /
    //        (target mass * nucleus mass) = 0.75
    let (sources, constants) = radial_atom_inputs(0.75);
    let a = model_acceleration(&sources, 0, &sources[0], &constants).unwrap();
    assert!((a.total.x).abs() < 1e-12);
    let result = reference_model_with_fixed_sources(
        &sources, 0, &constants,
        RetrySettings {
            duration: 1.0, initial_steps: 1000, max_steps: 1000,
        },
    ).unwrap();
    assert!((result.state.position.x - sources[0].position.x).abs() < 1e-10);
    assert!(result.state.velocity.x.abs() < 1e-10);
    assert!((result.state.mass - sources[0].mass).abs() < 1e-10);
    assert!((result.state.pulse - sources[0].pulse).abs() < 1e-10);
}

#[test]
fn radial_perturbations_accelerate_toward_balance() {
    let (inside, constants) = radial_atom_inputs(0.74);
    let (outside, _) = radial_atom_inputs(0.76);
    let ai = model_acceleration(&inside, 0, &inside[0], &constants).unwrap();
    let ao = model_acceleration(&outside, 0, &outside[0], &constants).unwrap();
    assert!(ai.total.x > 0.0, "inside must initially accelerate outward");
    assert!(ao.total.x < 0.0, "outside must initially accelerate inward");
    for sources in [inside, outside] {
        let result = reference_model_with_fixed_sources(
            &sources, 0, &constants,
            RetrySettings {
                duration: 0.2, initial_steps: 2000, max_steps: 2000,
            },
        ).unwrap();
        assert!(result.state.mass > 0.0 && result.state.pulse > 0.0);
        let energy = crate::energy_of(&result.state, &constants).unwrap();
        assert!((energy.total() - constants.cosmic_energy).abs() < 1e-9);
    }
}
