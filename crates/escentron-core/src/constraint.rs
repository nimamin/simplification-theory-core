//! Geometric building block for energy-constrained acceleration.
//!
//! This module is NOT an energy-depletion solver. A future dynamics algorithm
//! must determine which direction drains a reservoir and the permissible
//! forward acceleration from the reservoir's remaining energy.
use crate::Vector3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionalError {
    NonFiniteInput,
    ZeroDirection,
    NegativeLimit,
    NonFiniteResult,
}

/// Preserve acceleration perpendicular to and opposite to a specified
/// depletion direction, while limiting its forward component.
///
/// Given a direction d and acceleration a:
///   t = (a dot d) / (d dot d)
///   a_corrected = a - max(t - max_forward_factor, 0) * d
///
/// The limit refers to the coefficient multiplying d, not to a speed or
/// energy value. The direction need not be normalized; no sqrt is required.
///
/// A caller can set max_forward_factor=0 once a reservoir reaches its
/// numerical depletion threshold. The forbidden component then vanishes,
/// while orthogonal and reverse components remain intact.
///
/// IMPORTANT: choosing d and max_forward_factor requires a separate physical
/// energy-transfer and event-detection rule. No such rule is inferred here.
pub fn limit_forward_acceleration(
    acceleration: Vector3,
    depletion_direction: Vector3,
    max_forward_factor: f64,
) -> Result<Vector3, DirectionalError> {
    if !acceleration.is_finite()
        || !depletion_direction.is_finite()
        || !max_forward_factor.is_finite()
    {
        return Err(DirectionalError::NonFiniteInput);
    }
    if max_forward_factor < 0.0 {
        return Err(DirectionalError::NegativeLimit);
    }
    let d2 = depletion_direction.magnitude_squared();
    if !d2.is_finite() { return Err(DirectionalError::NonFiniteResult); }
    if d2 == 0.0 { return Err(DirectionalError::ZeroDirection); }

    let dot = acceleration.x * depletion_direction.x
        + acceleration.y * depletion_direction.y
        + acceleration.z * depletion_direction.z;
    if !dot.is_finite() { return Err(DirectionalError::NonFiniteResult); }
    let forward_factor = dot / d2;
    if !forward_factor.is_finite() {
        return Err(DirectionalError::NonFiniteResult);
    }
    if forward_factor <= max_forward_factor {
        return Ok(acceleration);
    }
    let excess = forward_factor - max_forward_factor;
    let corrected = Vector3::new(
        acceleration.x - excess * depletion_direction.x,
        acceleration.y - excess * depletion_direction.y,
        acceleration.z - excess * depletion_direction.z,
    );
    if !corrected.is_finite() { return Err(DirectionalError::NonFiniteResult); }
    Ok(corrected)
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/constraint.rs"));
}
