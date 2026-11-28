use super::{add, incident_fields, norm, scale, WorkVectors};
use crate::{EnergyBreakdown, FieldTotals, ParticleState, Vector3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct V2Constants {
    pub k_g: f64,
    pub k_l: f64,
    pub k_k: f64,
    pub c_g: f64,
    pub c_l: f64,
    pub energy_floor: f64,
    pub reference_field: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V2Error {
    InvalidInput,
    SingularField,
    ReservoirFloor,
    NonFinite,
    EnergyMismatch,
}

impl V2Constants {
    pub(super) fn validate(&self) -> Result<(), V2Error> {
        if [self.k_g, self.k_l, self.k_k, self.c_g, self.c_l, self.energy_floor,
            self.reference_field].iter().all(|x| x.is_finite() && *x > 0.0) {
            Ok(())
        } else { Err(V2Error::InvalidInput) }
    }
}

pub fn energies(state: &ParticleState, c: &V2Constants) -> Result<EnergyBreakdown, V2Error> {
    c.validate()?;
    state.validate().map_err(|_| V2Error::InvalidInput)?;
    let e = EnergyBreakdown {
        gravitational: c.k_g * state.mass * state.mass,
        levitational: c.k_l * state.pulse * state.pulse,
        kinetic: c.k_k * state.velocity.magnitude_squared(),
    };
    if !e.total().is_finite() { return Err(V2Error::NonFinite); }
    let tolerance = 32.0 * f64::EPSILON * c.energy_floor;
    if e.gravitational < c.energy_floor - tolerance ||
       e.levitational < c.energy_floor - tolerance { return Err(V2Error::ReservoirFloor); }
    Ok(e)
}

/// Advance one particle in externally supplied fields. Time is normalized
/// to the reference step; the time scaling and trapezoidal drift are provisional.
pub fn advance(state: &ParticleState, fields: FieldTotals, c: &V2Constants, dt: f64)
    -> Result<ParticleState, V2Error> {
    if !dt.is_finite() || dt <= 0.0 ||
        !fields.gravitational.0.is_finite() || !fields.levitational.0.is_finite() {
        return Err(V2Error::InvalidInput);
    }
    let e = energies(state, c)?;
    let raw = WorkVectors::from_fields(fields, e, c.reference_field, dt);
    if !raw.gravitational.is_finite() || !raw.levitational.is_finite()
        || !raw.total().is_finite() { return Err(V2Error::NonFinite); }
    let j = scale(state.velocity, c.k_k * norm(state.velocity));
    let (work, qg, ql) = raw.limited(j, e, c.energy_floor);
    let next_j = add(j, work.total());
    let kinetic = norm(next_j);
    let eg = e.gravitational - qg;
    let el = e.levitational - ql;
    let tolerance = 64.0 * f64::EPSILON * e.total();
    if eg < c.energy_floor - tolerance || el < c.energy_floor - tolerance {
        return Err(V2Error::ReservoirFloor);
    }
    let velocity = if kinetic == 0.0 { Vector3::ZERO } else {
        scale(scale(next_j, 1.0 / kinetic), (kinetic / c.k_k).sqrt())
    };
    let next = ParticleState {
        position: add(state.position, scale(add(state.velocity, velocity), dt * 0.5)),
        velocity,
        mass: (eg.max(c.energy_floor) / c.k_g).sqrt(),
        pulse: (el.max(c.energy_floor) / c.k_l).sqrt(),
    };
    let check = energies(&next, c)?;
    if (check.total() - e.total()).abs() > tolerance { return Err(V2Error::EnergyMismatch); }
    Ok(next)
}

/// All particles see the same old snapshot. No collision or bounce override.
pub fn advance_system(states: &[ParticleState], c: &V2Constants, dt: f64)
    -> Result<Vec<ParticleState>, V2Error> {
    states.iter().enumerate().map(|(i, state)| {
        advance(state, incident_fields(states, i, c)?, c, dt)
    }).collect()
}
