//! Explicit adapter: external source fields -> caller-defined raw acceleration
//! proposals -> energy-fraction scaling -> reference simulation.
//!
//! Only the target evolves. All other source particles are frozen during the
//! reference interval. This is NOT a self-consistent multi-body simulation.
use crate::{
    acceleration_from_fields, convergence_test_checked, fields_for_target_state,
    reference_run_checked, scale_gravitational_acceleration, scale_levitational_acceleration,
    AccelerationBreakdown, ConvergenceError, ConvergenceResult, ConvergenceSettings,
    ModelConstants, ParticleState, ReferenceError, ReferenceResult, ResponseError, ResponseLaw,
    RetrySettings, Vector3,
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

/// Query separate fields from other particles, convert each to an unscaled
/// acceleration proposal via a supplied law, and apply the target's energy
/// fraction alpha_g = E_g/E_c or alpha_l = E_l/E_c, exactly once.
///
/// The law returns RAW proposals; implementations must NOT apply the energy
/// response fractions themselves. No universal field-to-acceleration
/// conversion law has been established.
pub fn coupled_acceleration(
    sources: &[ParticleState],
    target_index: usize,
    current_target: &ParticleState,
    constants: &ModelConstants,
    law: &impl ResponseLaw,
) -> Result<AccelerationBreakdown, CouplingError> {
    let fields = fields_for_target_state(sources, target_index, current_target, constants)?;
    let proposed = acceleration_from_fields(current_target, &fields, law)?;
    let gravitational = scale_gravitational_acceleration(
        current_target, constants, proposed.gravitational,
    )?;
    let levitational = scale_levitational_acceleration(
        current_target, constants, proposed.levitational,
    )?;
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
}
