use crate::{convergence_test_checked, reference_run_checked, coupled_acceleration, CouplingError, ConvergenceError, ConvergenceResult, ConvergenceSettings, DirectFieldResponse, ModelConstants, ParticleState, ReferenceError, ReferenceResult, ResponseLaw, RetrySettings};

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
