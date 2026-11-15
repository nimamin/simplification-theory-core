//! Explicit adapter: external source fields -> caller-defined raw acceleration
//! proposals -> energy-fraction scaling -> reference simulation.
//!
//! Only the target evolves. All other source particles are frozen during the
//! reference interval. This is NOT a self-consistent multi-body simulation.
use crate::{
    acceleration_from_fields, convergence_test_checked, fields_for_target_state,
    reference_run_checked, gravitational_response_coefficient, levitational_response_coefficient,
    AccelerationBreakdown, ConvergenceError, ConvergenceResult, ConvergenceSettings,
    ModelConstants, ParticleState, ReferenceError, ReferenceResult, ResponseError, ResponseLaw,
    RetrySettings, Vector3, FieldTotals, DirectFieldResponse,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CouplingError {
    Field(crate::FieldError),
    Response(ResponseError),
}

impl From<crate::FieldError> for CouplingError {
    fn from(error: crate::FieldError) -> Self { Self::Field(error) }
}
impl From<ResponseError> for CouplingError {
    fn from(error: ResponseError) -> Self { Self::Response(error) }
}

/// Full auditable breakdown at a single position. The two field types
/// remain separate and are never numerically added.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CouplingTrace {
    pub fields: FieldTotals,
    /// Raw acceleration proposals from a supplied, explicitly chosen law.
    pub proposed: AccelerationBreakdown,
    pub gravitational_energy_fraction: f64,
    pub levitational_energy_fraction: f64,
    /// The two response-scaled accelerations and their vector resultant.
    pub applied: AccelerationBreakdown,
}

fn scale_vector(vector: Vector3, factor: f64) -> Vector3 {
    Vector3::new(vector.x * factor, vector.y * factor, vector.z * factor)
}

fn checked_acceleration(
    gravitational: Vector3,
    levitational: Vector3,
) -> Result<AccelerationBreakdown, CouplingError> {
    if !gravitational.is_finite() {
        return Err(CouplingError::Response(ResponseError::InvalidAcceleration("gravitational")));
    }
    if !levitational.is_finite() {
        return Err(CouplingError::Response(ResponseError::InvalidAcceleration("levitational")));
    }
    let total = Vector3::new(
        gravitational.x + levitational.x,
        gravitational.y + levitational.y,
        gravitational.z + levitational.z,
    );
    if !total.is_finite() {
        return Err(CouplingError::Response(ResponseError::NonFiniteTotal));
    }
    Ok(AccelerationBreakdown { gravitational, levitational, total })
}

/// Trace the complete *known* part of field-to-acceleration processing:
/// separate fields -> explicit raw law -> individual energy fractions
/// -> separately scaled acceleration vectors.
///
/// This function intentionally does not assume any missing physical
/// relation for conversion from fields to raw acceleration proposals.
pub fn coupling_trace(
    sources: &[ParticleState],
    target_index: usize,
    current_target: &ParticleState,
    constants: &ModelConstants,
    law: &impl ResponseLaw,
) -> Result<CouplingTrace, CouplingError> {
    let fields = fields_for_target_state(sources, target_index, current_target, constants)?;
    let proposed = acceleration_from_fields(current_target, &fields, law)?;
    let gravitational_energy_fraction =
        gravitational_response_coefficient(current_target, constants)?;
    let levitational_energy_fraction =
        levitational_response_coefficient(current_target, constants)?;
    let applied = checked_acceleration(
        scale_vector(proposed.gravitational, gravitational_energy_fraction),
        scale_vector(proposed.levitational, levitational_energy_fraction),
    )?;
    Ok(CouplingTrace {
        fields,
        proposed,
        gravitational_energy_fraction,
        levitational_energy_fraction,
        applied,
    })
}

/// Lightweight path for the reference solver. The independent field response
/// and the two energy fractions are applied exactly once.
pub fn coupled_acceleration(
    sources: &[ParticleState],
    target_index: usize,
    current_target: &ParticleState,
    constants: &ModelConstants,
    law: &impl ResponseLaw,
) -> Result<AccelerationBreakdown, CouplingError> {
    Ok(coupling_trace(sources, target_index, current_target, constants, law)?.applied)
}

/// Run the reference integrator with a stationary environment and a supplied
/// physical response law. On depletion it restarts from the original target.
pub fn reference_with_fixed_sources(
    sources: &[ParticleState],
    target_index: usize,
    constants: &ModelConstants,
    settings: RetrySettings,
    law: &impl ResponseLaw,
) -> Result<ReferenceResult, ReferenceError> {
    let target = *sources.get(target_index).ok_or(
        ReferenceError::Field(crate::FieldError::InvalidTargetIndex)
    )?;
    reference_run_checked(&target, constants, settings, |current, _time| {
        let a = coupled_acceleration(sources, target_index, current, constants, law)
            .map_err(|err| match err {
                CouplingError::Field(e) => ReferenceError::Field(e),
                CouplingError::Response(e) => ReferenceError::Response(e),
            })?;
        Ok((a.gravitational, a.levitational))
    })
}

/// Run timestep convergence comparisons over the same fixed external sources.
pub fn convergence_with_fixed_sources(
    sources: &[ParticleState],
    target_index: usize,
    constants: &ModelConstants,
    settings: ConvergenceSettings,
    law: &impl ResponseLaw,
) -> Result<ConvergenceResult, ConvergenceError> {
    let target = *sources.get(target_index).ok_or(
        ConvergenceError::ReferenceFailed {
            steps: settings.initial_steps,
            error: ReferenceError::Field(crate::FieldError::InvalidTargetIndex),
        }
    )?;
    convergence_test_checked(&target, constants, settings, |current, _time| {
        let a = coupled_acceleration(sources, target_index, current, constants, law)
            .map_err(|err| match err {
                CouplingError::Field(e) => ReferenceError::Field(e),
                CouplingError::Response(e) => ReferenceError::Response(e),
            })?;
        Ok((a.gravitational, a.levitational))
    })
}

/// The confirmed model acceleration for the target under other sources:
/// a_g = (E_g/E_c) * f_g; a_l = (E_l/E_c) * f_l.
/// The two acceleration vectors, unlike their source field types, can sum.
pub fn model_acceleration(
    sources: &[ParticleState],
    target_index: usize,
    current_target: &ParticleState,
    constants: &ModelConstants,
) -> Result<AccelerationBreakdown, CouplingError> {
    coupled_acceleration(
        sources, target_index, current_target, constants, &DirectFieldResponse,
    )
}

/// Reference trajectory using the confirmed field-to-acceleration relation.
pub fn reference_model_with_fixed_sources(
    sources: &[ParticleState],
    target_index: usize,
    constants: &ModelConstants,
    settings: RetrySettings,
) -> Result<ReferenceResult, ReferenceError> {
    reference_with_fixed_sources(
        sources, target_index, constants, settings, &DirectFieldResponse,
    )
}

/// Timestep refinement with the confirmed field-to-acceleration relation.
pub fn convergence_model_with_fixed_sources(
    sources: &[ParticleState],
    target_index: usize,
    constants: &ModelConstants,
    settings: ConvergenceSettings,
) -> Result<ConvergenceResult, ConvergenceError> {
    convergence_with_fixed_sources(
        sources, target_index, constants, settings, &DirectFieldResponse,
    )
}

#[cfg(test)]
mod tests {
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

}
