//! Independent *continuous-limit* benchmarks for constant fields.
//!
//! These formulas do not call the discrete step implementation and provide
//! analytically derived reference checks. They are model hypotheses, not
//! experimental evidence. Only explicitly documented 1D cases are supported.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnalyticError {
    InvalidInput,
    PrecisionLimit,
}

impl std::fmt::Display for AnalyticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AnalyticError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SingleFieldExact {
    pub velocity: f64,
    pub remaining_field_energy: f64,
    pub kinetic_energy: f64,
}

/// Exact continuous-time response along a single fixed field axis.
///
/// Assumes a single active reservoir E>0, k_k>0, E_c>0,
/// dv/dt = field_strength * E/E_c, E + k_k*v^2 = constant.
/// The field strength is a SIGNED 1D scalar. The opposite reservoir stays
/// fixed and does not participate in energy exchange.
///
/// Let C=E_0+k_k*v_0², U=sqrt(C/k_k),
/// z=atanh(v_0/U)+(field_strength/E_c)*sqrt(C*k_k)*duration.
/// Then v=U*tanh(z), E=C/cosh(z)². No timestepping and no clamping.
/// Extremely large z can underflow to exactly zero in f64; this returns
/// PrecisionLimit rather than inventing a positive epsilon.
pub fn single_field_constant_exact(
    initial_velocity: f64,
    initial_field_energy: f64,
    field_strength: f64,
    duration: f64,
    kinetic_coefficient: f64,
    cosmic_energy: f64,
) -> Result<SingleFieldExact, AnalyticError> {
    if !initial_velocity.is_finite()
        || !initial_field_energy.is_finite()
        || !field_strength.is_finite()
        || !duration.is_finite()
        || !kinetic_coefficient.is_finite()
        || !cosmic_energy.is_finite()
        || initial_field_energy <= 0.0
        || duration < 0.0
        || kinetic_coefficient <= 0.0
        || cosmic_energy <= 0.0
    {
        return Err(AnalyticError::InvalidInput);
    }
    let c = initial_field_energy
        + kinetic_coefficient * initial_velocity * initial_velocity;
    let u = (c / kinetic_coefficient).sqrt();
    let ratio = initial_velocity / u;
    if !u.is_finite() || !ratio.is_finite() || ratio.abs() >= 1.0 {
        return Err(AnalyticError::PrecisionLimit);
    }
    let z = ratio.atanh()
        + (field_strength / cosmic_energy)
            * (c * kinetic_coefficient).sqrt() * duration;
    if !z.is_finite() {
        return Err(AnalyticError::PrecisionLimit);
    }
    let ch = z.cosh();
    if !ch.is_finite() {
        return Err(AnalyticError::PrecisionLimit);
    }
    let energy = (c / ch) / ch;
    let velocity = u * z.tanh();
    let kinetic_energy = kinetic_coefficient * velocity * velocity;
    if !energy.is_finite() || energy <= 0.0
        || !velocity.is_finite() || !kinetic_energy.is_finite()
    {
        return Err(AnalyticError::PrecisionLimit);
    }
    Ok(SingleFieldExact {
        velocity,
        remaining_field_energy: energy,
        kinetic_energy,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignedFieldsExact {
    pub gravitational_energy: f64,
    pub levitational_energy: f64,
}

/// Analytical energy-versus-distance invariant, for two co-directed
/// positive constant fields along +x, positive displacement x, and a
/// continuous trajectory whose acceleration attribution stays co-directed.
///
/// dE_g/dx = -2*k_k*(f_g/E_c)*E_g
/// dE_l/dx = -2*k_k*(f_l/E_c)*E_l
/// Hence E_g(x)=E_g(0)*exp(-2*k_k*f_g*x/E_c) and similarly for E_l.
///
/// This is NOT a closed-form solution for position as a function of time.
/// Reverse trajectories and opposing field directions require separate
/// treatment; no general solution is claimed.
pub fn aligned_fields_energy_at_distance(
    initial_gravitational_energy: f64,
    initial_levitational_energy: f64,
    gravitational_field: f64,
    levitational_field: f64,
    displacement: f64,
    kinetic_coefficient: f64,
    cosmic_energy: f64,
) -> Result<AlignedFieldsExact, AnalyticError> {
    let inputs = [
        initial_gravitational_energy, initial_levitational_energy,
        gravitational_field, levitational_field, displacement,
        kinetic_coefficient, cosmic_energy,
    ];
    if !inputs.iter().all(|v| v.is_finite())
        || initial_gravitational_energy <= 0.0
        || initial_levitational_energy <= 0.0
        || gravitational_field <= 0.0
        || levitational_field <= 0.0
        || displacement < 0.0
        || kinetic_coefficient <= 0.0
        || cosmic_energy <= 0.0
    {
        return Err(AnalyticError::InvalidInput);
    }
    let factor = -2.0 * kinetic_coefficient * displacement / cosmic_energy;
    let eg = initial_gravitational_energy
        * (factor * gravitational_field).exp();
    let el = initial_levitational_energy
        * (factor * levitational_field).exp();
    if !eg.is_finite() || !el.is_finite() || eg <= 0.0 || el <= 0.0 {
        return Err(AnalyticError::PrecisionLimit);
    }
    Ok(AlignedFieldsExact {
        gravitational_energy: eg,
        levitational_energy: el,
    })
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/analytic.rs"));
}
