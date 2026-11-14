//! Experimental computational core for Simplification Theory.
//!
//! This crate currently defines the Escentron data model and energy bookkeeping.
//! It does not yet integrate equations of motion or claim physical validation.

mod allocation;
mod budget;
mod constants;
mod energy;
mod field;
mod particle;
mod response;
mod step;
mod validation;
mod vector;

pub use allocation::{acceleration_shares, allocate_kinetic_change, AccelerationShares, AllocationError};
pub use budget::{transfer_energy, EnergyReservoir, TransferError};
pub use constants::ModelConstants;
pub use energy::{energy_of, EnergyBreakdown};
pub use field::{field_at, fields_at, FieldError, FieldTotals, GravitationalField, LevitationalField, SourceFields};
pub use particle::ParticleState;
pub use response::{acceleration_from_fields, gravitational_response_coefficient, levitational_response_coefficient, scale_gravitational_acceleration, scale_levitational_acceleration, AccelerationBreakdown, ResponseError, ResponseLaw};
pub use step::{propose_step, StepError, StepProposal};
pub use validation::ValidationError;
pub use vector::Vector3;

/// Version of this Rust implementation.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_has_a_version() {
        assert!(!CORE_VERSION.is_empty());
    }
}
