//! Restart-from-initial-state reference integration with decreasing timestep.
//!
//! A depleted reservoir makes an attempt invalid. The attempt is discarded,
//! the original particle state is restored, and the entire interval is
//! evaluated again with a smaller timestep. This is a *reference diagnostic*,
//! not a validated physical model or a substitute for convergence testing.
use crate::{
    energy_of, propose_step, ModelConstants, ParticleState, StepError, TransferError, ValidationError,
    Vector3,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetrySettings {
    pub duration: f64,
    pub initial_steps: usize,
    pub max_steps: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferenceError {
    InvalidSettings,
    InvalidInput(ValidationError),
    UnsupportedConversionCoefficient,
    Step(StepError),
    RetryLimitReached { last_step_error: StepError, attempted_steps: usize },
    NonFiniteState,
    Field(crate::FieldError),
    Response(crate::ResponseError),
}

impl From<crate::FieldError> for ReferenceError {
    fn from(error: crate::FieldError) -> Self { Self::Field(error) }
}
impl From<crate::ResponseError> for ReferenceError {
    fn from(error: crate::ResponseError) -> Self { Self::Response(error) }
}

impl std::fmt::Display for ReferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ReferenceError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReferenceResult {
    pub state: ParticleState,
    pub accepted_steps: usize,
    pub retries: usize,
    pub time_step: f64,
}

/// Integrate one target for a fixed duration using a caller-supplied
/// acceleration model. `acceleration_at(state,time)` must be deterministic
/// and free of mutable cross-attempt state. Both returned vectors are
/// accelerations, NOT gravitational/levitational fields.
///
/// The proposal uses v'=v+a*dt and a semi-implicit position proposal
/// x'=x+v'*dt. Each successful step converts its energy accounting back
/// into m'=E_g'/k_g and p'=E_l'/k_l. The next step recomputes accelerations
/// from the *updated* state via the provided callback.
///
/// On depletion/overspend, the WHOLE trial is discarded; step count doubles
/// and the same initial state/time are used again. Other errors stop early.
/// Retry count and work are bounded by max_steps.
///
/// A successful run does not certify timestep convergence or physical
/// stability, and no self-consistent induction law is supplied here.
pub fn reference_run<F>(
    initial: &ParticleState,
    constants: &ModelConstants,
    settings: RetrySettings,
    acceleration_at: F,
) -> Result<ReferenceResult, ReferenceError>
where
    F: Fn(&ParticleState, f64) -> (Vector3, Vector3),
{
    reference_run_checked(initial, constants, settings, |state, time| {
        Ok(acceleration_at(state, time))
    })
}

/// A fallible version of the reference runner. Invalid field/response
/// evaluations fail explicitly instead of being converted to bogus vectors.
pub fn reference_run_checked<F>(
    initial: &ParticleState,
    constants: &ModelConstants,
    settings: RetrySettings,
    acceleration_at: F,
) -> Result<ReferenceResult, ReferenceError>
where
    F: Fn(&ParticleState, f64) -> Result<(Vector3, Vector3), ReferenceError>,
{
    initial.validate().map_err(ReferenceError::InvalidInput)?;
    constants.validate().map_err(ReferenceError::InvalidInput)?;
    if constants.k_g <= 0.0 || constants.k_l <= 0.0 || constants.k_k <= 0.0 {
        return Err(ReferenceError::UnsupportedConversionCoefficient);
    }
    if !settings.duration.is_finite() || settings.duration <= 0.0
        || settings.initial_steps == 0
        || settings.max_steps < settings.initial_steps
    {
        return Err(ReferenceError::InvalidSettings);
    }
    let mut steps = settings.initial_steps;
    let mut retries = 0;
    loop {
        let dt = settings.duration / steps as f64;
        if dt <= 0.0 || !dt.is_finite() {
            return Err(ReferenceError::InvalidSettings);
        }
        let mut state = *initial;
        let mut failure = None;
        for index in 0..steps {
            let time = index as f64 * dt;
            let (g, l) = acceleration_at(&state, time)?;
            let proposal = match propose_step(&state, constants, g, l, dt) {
                Ok(proposal) => proposal,
                Err(err) => { failure = Some(err); break; }
            };

            let next_position = Vector3::new(
                state.position.x + proposal.proposed_velocity.x * dt,
                state.position.y + proposal.proposed_velocity.y * dt,
                state.position.z + proposal.proposed_velocity.z * dt,
            );
            let next_state = ParticleState {
                position: next_position,
                velocity: proposal.proposed_velocity,
                mass: proposal.proposed_energies.gravitational / constants.k_g,
                pulse: proposal.proposed_energies.levitational / constants.k_l,
            };
            if !next_position.is_finite() {
                return Err(ReferenceError::NonFiniteState);
            }
            // A valid existing Escentron always has strictly positive m,p.
            // Degenerate or underflowed states are treated as depletion.
            if next_state.mass <= 0.0 || next_state.pulse <= 0.0 {
                failure = Some(StepError::Transfer(TransferError::NonPositiveFieldEnergy));
                break;
            }
            if let Err(err) = next_state.validate() {
                return Err(ReferenceError::InvalidInput(err));
            }
            state = next_state;
        }
        if let Some(err) = failure {
            let is_depletion = matches!(
                err,
                StepError::Transfer(
                    TransferError::InsufficientSourceEnergy
                        | TransferError::InsufficientKineticEnergy
                        | TransferError::NonPositiveFieldEnergy
                )
            );
            if !is_depletion { return Err(ReferenceError::Step(err)); }
            if steps > settings.max_steps / 2 {
                return Err(ReferenceError::RetryLimitReached {
                    last_step_error: err,
                    attempted_steps: steps,
                });
            }
            steps *= 2;
            retries += 1;
            continue;
        }
        // Recompute the conserved-energy diagnostic after reconstruction.
        let final_energy = energy_of(&state, constants).map_err(ReferenceError::InvalidInput)?;
        let tolerance = 1e-10 * constants.cosmic_energy.max(1.0);
        if (final_energy.total() - constants.cosmic_energy).abs() > tolerance {
            return Err(ReferenceError::Step(StepError::EnergyMismatch));
        }
        return Ok(ReferenceResult {
            state,
            accepted_steps: steps,
            retries,
            time_step: dt,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn constants() -> ModelConstants {
        ModelConstants {
            g: 1.0, k_g: 1.0, k_l: 1.0, k_k: 1.0, cosmic_energy: 10.0,
        }
    }
    fn state() -> ParticleState {
        ParticleState {
            position: Vector3::ZERO,
            velocity: Vector3::new(1.0, 0.0, 0.0),
            mass: 6.0,
            pulse: 3.0,
        }
    }
    fn settings() -> RetrySettings {
        RetrySettings { duration: 1.0, initial_steps: 1, max_steps: 8192 }
    }

    #[test]
    fn retries_from_original_state_when_coarse_step_depletes() {
        // Test-only acceleration with gravitational response proportional
        // to the remaining source reservoir; not the theory's full field law.
        let result = reference_run(&state(), &constants(), settings(), |s, _| {
            (Vector3::new(s.mass, 0.0, 0.0), Vector3::ZERO)
        }).unwrap();
        assert!(result.retries > 0);
        assert!(result.accepted_steps > 1);
        assert!(result.state.mass > 0.0);
        assert!(result.state.pulse > 0.0);
        let e = energy_of(&result.state, &constants()).unwrap();
        assert!((e.total() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn a_stable_trial_needs_no_retry() {
        let result = reference_run(&state(), &constants(), settings(), |_, _| {
            (Vector3::ZERO, Vector3::ZERO)
        }).unwrap();
        assert_eq!(result.retries, 0);
        assert_eq!(result.accepted_steps, 1);
        assert_eq!(result.state.mass, 6.0);
        assert_eq!(result.state.pulse, 3.0);
    }

    #[test]
    fn retries_have_a_finite_limit() {
        let options = RetrySettings { duration: 1.0, initial_steps: 1, max_steps: 2 };
        let result = reference_run(&state(), &constants(), options, |_, _| {
            (Vector3::new(100.0, 0.0, 0.0), Vector3::ZERO)
        });
        assert!(matches!(result, Err(ReferenceError::RetryLimitReached { .. })));
    }

    #[test]
    fn other_invalid_steps_do_not_trigger_retries() {
        let result = reference_run(&state(), &constants(), settings(), |_, _| {
            (Vector3::new(f64::NAN, 0.0, 0.0), Vector3::ZERO)
        });
        assert!(matches!(result, Err(ReferenceError::Step(_))));
    }

    #[test]
    fn rejects_zero_coefficient_for_energy_to_mass_conversion() {
        let mut c = constants();
        c.k_g = 0.0;
        assert_eq!(
            reference_run(&state(), &c, settings(), |_, _| {
                (Vector3::ZERO, Vector3::ZERO)
            }),
            Err(ReferenceError::UnsupportedConversionCoefficient)
        );
    }
}
