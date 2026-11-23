//! Allocate a proposed kinetic-energy change between acceleration channels.
//!
//! Fields are never combined here. The inputs are acceleration vectors
//! obtained separately from the gravitational and levitational responses.
//! This is a geometric allocation rule; it is not a timestep integrator.
use crate::Vector3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    NonFiniteInput,
    NonFiniteResult,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccelerationShares {
    /// Acceleration after vector addition of the two response channels.
    pub total: Vector3,
    /// Fraction of a subsequent kinetic-energy change assigned to gravity.
    pub gravitational: f64,
    /// Fraction assigned to levitation. Shares sum to one for nonzero total.
    pub levitational: f64,
}

/// Compute the relative projections onto total acceleration without sqrt.
///
/// Let A = a_g + a_l and d_g = a_g dot A, d_l = a_l dot A.
/// Since d_g + d_l = |A|², the positive projection ratios follow directly:
///   w_g = d_g / (d_g + d_l), w_l = d_l / (d_g + d_l).
///
/// If one projection opposes A (negative dot product), the opposing channel
/// is assigned zero; the other channel receives all of the *net* effect.
/// In perfect cancellation A=0, both shares are zero.
///
/// This function does not move energy from one non-kinetic reservoir to the
/// other. It only labels how a subsequent signed kinetic-energy delta is
/// attributed to its source or destination reservoirs.
pub fn acceleration_shares(
    gravitational: Vector3,
    levitational: Vector3,
) -> Result<AccelerationShares, AllocationError> {
    if !gravitational.is_finite() || !levitational.is_finite() {
        return Err(AllocationError::NonFiniteInput);
    }
    let total = crate::equations::resultant(gravitational, levitational);
    if !total.is_finite() {
        return Err(AllocationError::NonFiniteResult);
    }
    use crate::equations::dot;
    let total_squared = dot(total, total);
    if !total_squared.is_finite() {
        return Err(AllocationError::NonFiniteResult);
    }
    if total_squared == 0.0 {
        return Ok(AccelerationShares {
            total,
            gravitational: 0.0,
            levitational: 0.0,
        });
    }

    let g = dot(gravitational, total);
    let l = dot(levitational, total);
    if !g.is_finite() || !l.is_finite() {
        return Err(AllocationError::NonFiniteResult);
    }

    if g > 0.0 && l > 0.0 && !(g / total_squared).is_finite() {
        return Err(AllocationError::NonFiniteResult);
    }
    let g_share = crate::equations::gravity_share(g, l, total_squared);
    let l_share = 1.0 - g_share;
    Ok(AccelerationShares {
        total,
        gravitational: g_share,
        levitational: l_share,
    })
}

/// Divide a *separately computed* signed kinetic-energy change between
/// the two reservoirs according to the acceleration shares.
/// No reservoir is modified here; use transfer_energy for that step.
pub fn allocate_kinetic_change(
    shares: AccelerationShares,
    delta_kinetic: f64,
) -> Result<(f64, f64), AllocationError> {
    if !delta_kinetic.is_finite()
        || !shares.gravitational.is_finite()
        || !shares.levitational.is_finite()
        || shares.gravitational < 0.0
        || shares.levitational < 0.0
        || shares.gravitational > 1.0
        || shares.levitational > 1.0
        || (shares.gravitational + shares.levitational - 1.0).abs() > 1e-12
            && (shares.gravitational != 0.0 || shares.levitational != 0.0)
    {
        return Err(AllocationError::NonFiniteInput);
    }
    let (g, l) = crate::equations::split_change(delta_kinetic, shares.gravitational);
    if !g.is_finite() || !l.is_finite() {
        return Err(AllocationError::NonFiniteResult);
    }
    // If the total acceleration is zero, only zero kinetic change is valid.
    if shares.gravitational == 0.0 && shares.levitational == 0.0 {
        if delta_kinetic != 0.0 { return Err(AllocationError::NonFiniteInput); }
        return Ok((0.0, 0.0));
    }
    Ok((g, l))
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/allocation.rs"));
}
