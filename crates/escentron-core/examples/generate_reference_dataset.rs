//! Generate a small, reproducible *model-reference* dataset in CSV format.
//!
//! Usage:
//!   cargo run --release --example generate_reference_dataset > reference-dataset.csv
//!
//! Includes only samples passing the convergence test. Uses the model's
//! confirmed a_g=(E_g/E_c)f_g and a_l=(E_l/E_c)f_l, but still relies on the
//! experimental energy-allocation and position-integration rules.
//! These are not measured or experimentally validated physics data.
use escentron_core::{
    convergence_model_with_fixed_sources, energy_of, model_acceleration, propose_step,
    ConvergenceSettings, ModelConstants, ParticleState, Vector3,
};

fn main() {
    let constants = ModelConstants {
        g: 1.0,
        k_g: 1.0,
        k_l: 1.0,
        k_k: 1.0,
        cosmic_energy: 10.0,
    };
    let source = ParticleState {
        position: Vector3::new(10.0, 0.0, 0.0),
        velocity: Vector3::ZERO,
        mass: 4.0,
        pulse: 6.0,
    };
    // A second non-collinear field source makes the two acceleration
    // vectors potentially non-parallel, exercising 3D allocation.
    let second_source = ParticleState {
        position: Vector3::new(0.0, 12.0, -4.0),
        velocity: Vector3::ZERO,
        mass: 5.0,
        pulse: 4.0,
    };
    let settings = ConvergenceSettings {
        duration: 1.0,
        initial_steps: 8,
        max_steps: 8192,
        absolute_tolerance: 1e-4,
        relative_tolerance: 1e-4,
    };

    // Include a near-source case with coarse-step energy overspend.
    // All positions, units, constants and tolerances are provisional.
    let positions = [
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(-4.0, 2.0, 0.0),
        Vector3::new(3.0, -3.0, 1.0),
        Vector3::new(9.0, 0.0, 0.0),
    ];
    let velocities = [
        Vector3::new(0.1, 0.0, 0.0),
        Vector3::new(-0.1, 0.05, 0.0),
        Vector3::new(0.02, 0.03, -0.08),
    ];
    println!(
        "sample_id,initial_px,initial_py,initial_pz,initial_vx,initial_vy,initial_vz,initial_mass,initial_pulse,initial_eg,initial_el,initial_ek,ag_x,ag_y,ag_z,al_x,al_y,al_z,coarse_vx,coarse_vy,coarse_vz,coarse_valid,reference_px,reference_py,reference_pz,reference_vx,reference_vy,reference_vz,reference_mass,reference_pulse,reference_eg,reference_el,reference_ek,reference_steps,dt,position_difference,velocity_difference,mass_difference,pulse_difference"
    );

    let mut sample = 0usize;
    let mut rejected = 0usize;
    for position in positions {
        for velocity in velocities {
            for mass in [2.0, 4.0, 6.0] {
                let kinetic = constants.k_k * velocity.magnitude_squared();
                let pulse = (constants.cosmic_energy - mass) - kinetic;
                let target = ParticleState {
                    position, velocity, mass, pulse,
                };
                let sources = [target, source, second_source];
                let reference = match convergence_model_with_fixed_sources(
                    &sources, 0, &constants, settings,
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        rejected += 1;
                        eprintln!("Skipped scenario ({position:?}, {velocity:?}, m={mass}): {error:?}");
                        continue;
                    }
                };
                let acceleration = model_acceleration(
                    &sources, 0, &target, &constants,
                ).expect("initial sample must produce finite acceleration");
                let coarse = Vector3::new(
                    velocity.x + acceleration.total.x * settings.duration,
                    velocity.y + acceleration.total.y * settings.duration,
                    velocity.z + acceleration.total.z * settings.duration,
                );
                let coarse_valid = propose_step(
                    &target, &constants,
                    acceleration.gravitational,
                    acceleration.levitational,
                    settings.duration,
                ).is_ok();
                let initial_energy = energy_of(&target, &constants())
                    .expect("valid target energy");
                let final_state = reference.reference.state;
                let final_energy = energy_of(&final_state, &constants())
                    .expect("valid reference energy");
                let d = reference.final_differences;
                println!(
                    "{sample},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    position.x, position.y, position.z,
                    velocity.x, velocity.y, velocity.z,
                    mass, pulse,
                    initial_energy.gravitational,
                    initial_energy.levitational,
                    initial_energy.kinetic,
                    acceleration.gravitational.x,
                    acceleration.gravitational.y,
                    acceleration.gravitational.z,
                    acceleration.levitational.x,
                    acceleration.levitational.y,
                    acceleration.levitational.z,
                    coarse.x, coarse.y, coarse.z, coarse_valid,
                    final_state.position.x,
                    final_state.position.y,
                    final_state.position.z,
                    final_state.velocity.x,
                    final_state.velocity.y,
                    final_state.velocity.z,
                    final_state.mass, final_state.pulse,
                    final_energy.gravitational,
                    final_energy.levitational,
                    final_energy.kinetic,
                    reference.reference.accepted_steps,
                    reference.reference.time_step,
                    d.position, d.velocity, d.mass, d.pulse,
                );
                sample += 1;
            }
        }
    }
    eprintln!("Reference dataset: {sample} converged, {rejected} rejected");
    assert!(sample > 0, "no converged reference data produced");
}
