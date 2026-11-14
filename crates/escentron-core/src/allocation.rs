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
    let total = Vector3::new(
        gravitational.x + levitational.x,
        gravitational.y + levitational.y,
        gravitational.z + levitational.z,
    );
    if !total.is_finite() {
        return Err(AllocationError::NonFiniteResult);
    }
    let dot = |a: Vector3, b: Vector3| a.x*b.x + a.y*b.y + a.z*b.z;
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

    let (g_share, l_share) = if g <= 0.0 {
        (0.0, 1.0)
    } else if l <= 0.0 {
        (1.0, 0.0)
    } else {
        // Both projections point along the resultant. No square root,
        // normalization of direction, angle or trigonometric function.
        let share = g / total_squared;
        if !share.is_finite() {
            return Err(AllocationError::NonFiniteResult);
        }
        let share = share.clamp(0.0, 1.0);
        (share, 1.0 - share)
    };
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
    let g = delta_kinetic * shares.gravitational;
    let l = delta_kinetic - g;
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
    use super::*;

    fn v(x: f64, y: f64) -> Vector3 { Vector3::new(x, y, 0.0) }
    fn eq(a: f64, b: f64) { assert!((a - b).abs() < 1e-12, "{a} != {b}"); }

    #[test]
    fn aligned_accelerations_share_by_projected_effect() {
        let s = acceleration_shares(v(3.0, 0.0), v(1.0, 0.0)).unwrap();
        eq(s.gravitational, 0.75);
        eq(s.levitational, 0.25);
        assert_eq!(s.total, v(4.0, 0.0));
        assert_eq!(allocate_kinetic_change(s, 20.0).unwrap(), (15.0, 5.0));
    }

    #[test]
    fn opposing_collinear_acceleration_is_cancelled() {
        let s = acceleration_shares(v(4.0, 0.0), v(-2.0, 0.0)).unwrap();
        assert_eq!(s.total, v(2.0, 0.0));
        assert_eq!((s.gravitational, s.levitational), (1.0, 0.0));
        assert_eq!(allocate_kinetic_change(s, 10.0).unwrap(), (10.0, 0.0));
    }

    #[test]
    fn weaker_gravity_is_cancelled_by_levitation() {
        let s = acceleration_shares(v(-1.0, 0.0), v(3.0, 0.0)).unwrap();
        assert_eq!((s.gravitational, s.levitational), (0.0, 1.0));
    }

    #[test]
    fn angled_positive_projections_split_energy() {
        let s = acceleration_shares(v(2.0, 0.0), v(0.0, 2.0)).unwrap();
        eq(s.gravitational, 0.5);
        eq(s.levitational, 0.5);
    }

    #[test]
    fn angled_opposing_projection_gets_no_share() {
        let s = acceleration_shares(v(3.0, 0.0), v(-2.0, 1.0)).unwrap();
        assert_eq!(s.total, v(1.0, 1.0));
        assert_eq!((s.gravitational, s.levitational), (1.0, 0.0));
    }

    #[test]
    fn perfect_cancellation_has_zero_shares() {
        let s = acceleration_shares(v(2.0, 1.0), v(-2.0, -1.0)).unwrap();
        assert_eq!((s.gravitational, s.levitational), (0.0, 0.0));
        assert_eq!(allocate_kinetic_change(s, 0.0).unwrap(), (0.0, 0.0));
        assert_eq!(allocate_kinetic_change(s, 1.0), Err(AllocationError::NonFiniteInput));
    }

    #[test]
    fn reverse_kinetic_change_retains_allocation() {
        let s = acceleration_shares(v(1.0, 0.0), v(3.0, 0.0)).unwrap();
        assert_eq!(allocate_kinetic_change(s, -8.0).unwrap(), (-2.0, -6.0));
    }

    #[test]
    fn rejects_nonfinite_accelerations() {
        assert_eq!(
            acceleration_shares(v(f64::INFINITY, 0.0), v(1.0, 0.0)),
            Err(AllocationError::NonFiniteInput)
        );
    }
}
