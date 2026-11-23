//! Experimental computational core for Simplification Theory.
//!
//! This crate currently defines the Escentron data model and energy bookkeeping.
//! It does not yet integrate equations of motion or claim physical validation.

mod allocation;
mod analytic;
mod budget;
mod constants;
mod constraint;
mod coupling;
mod energy;
mod field;
mod particle;
mod response;
mod step;
mod work_path;
mod equations;
mod fixed_sources;
mod convergence;
mod reference;
mod validation;
mod vector;

pub use allocation::{acceleration_shares, allocate_kinetic_change, AccelerationShares, AllocationError};
pub use analytic::{single_field_constant_exact, aligned_fields_energy_at_distance, AnalyticError, SingleFieldExact, AlignedFieldsExact};
pub use budget::{transfer_energy, EnergyReservoir, TransferError};
pub use constants::ModelConstants;
pub use constraint::{limit_forward_acceleration, DirectionalError};
pub use convergence::{convergence_test, convergence_test_checked, state_differences, ConvergenceError, ConvergenceResult, ConvergenceSettings, StateDifferences};
pub use coupling::{coupling_trace, CouplingTrace, coupled_acceleration, model_acceleration, CouplingError};
pub use energy::{energy_of, EnergyBreakdown};
pub use field::{field_at, fields_at, fields_for_particle, fields_for_target_state, inner_radius, INNER_RADIUS_RATIO, FieldError, FieldTotals, GravitationalField, LevitationalField, SourceFields};
pub use particle::ParticleState;
pub use reference::{reference_run, reference_run_checked, ReferenceError, ReferenceResult, RetrySettings};
pub use response::{acceleration_from_fields, gravitational_response_coefficient, levitational_response_coefficient, scale_gravitational_acceleration, scale_levitational_acceleration, AccelerationBreakdown, DirectFieldResponse, ResponseError, ResponseLaw};
pub use step::{propose_step, StepError, StepProposal};
pub use validation::ValidationError;
pub use vector::Vector3;
pub use work_path::{constant_field_work_path, WorkPathEnergies, WorkPathError};

pub use fixed_sources::{convergence_model_with_fixed_sources, reference_model_with_fixed_sources, convergence_with_fixed_sources, reference_with_fixed_sources};

/// Version of this Rust implementation.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/lib.rs"));
}
