//! Experimental direct integration of mass, pulse and velocity.
use super::{add, dot, norm, scale, V2Constants, V2Error};
use crate::{EnergyBreakdown, FieldTotals, ParticleState};

#[derive(Debug, Clone, Copy)]
pub struct DirectModel {
    pub constants: V2Constants,
    pub initial_cosmic_energy: f64,
    pub mass_max: f64,
    pub pulse_max: f64,
    pub mass_min: f64,
    pub pulse_min: f64,
}

impl DirectModel {
    pub fn new(initial: &ParticleState, constants: V2Constants) -> Result<Self, V2Error> {
        constants.validate()?;
        initial.validate().map_err(|_| V2Error::InvalidInput)?;
        let ec = observed_energy(initial, &constants).total();
        if !ec.is_finite() || ec <= 0.0 { return Err(V2Error::InvalidInput); }
        Ok(Self {
            constants,
            initial_cosmic_energy: ec,
            mass_max: (ec / constants.k_g).sqrt(),
            pulse_max: (ec / constants.k_l).sqrt(),
            mass_min: (constants.energy_floor / constants.k_g).sqrt(),
            pulse_min: (constants.energy_floor / constants.k_l).sqrt(),
        })
    }

    pub fn advance(&self, state: &ParticleState, fields: FieldTotals, dt: f64)
        -> Result<ParticleState, V2Error> {
        if !dt.is_finite() || dt <= 0.0 { return Err(V2Error::InvalidInput); }
        let c = self.constants;
        state.validate().map_err(|_| V2Error::InvalidInput)?;
        if state.mass < self.mass_min || state.pulse < self.pulse_min {
            return Err(V2Error::ReservoirFloor);
        }
        let work_g = scale(fields.gravitational.0, c.k_w * (state.mass / self.mass_max) / c.reference_field);
        let work_l = scale(fields.levitational.0, c.k_w * (state.pulse / self.pulse_max) / c.reference_field);
        let work_k = add(work_g, work_l);
        let length = norm(work_k);
        // Provisional signed scalar work convention: project onto the resultant.
        // Reverse the reservoir exchange during braking.
        let (mass_rate, pulse_rate) = if length == 0.0 { (0.0, 0.0) } else {
            let unit = scale(work_k, 1.0 / length);
            let sign = if dot(state.velocity, work_k) < 0.0 { 1.0 } else { -1.0 };
            (sign * dot(work_g, unit), sign * dot(work_l, unit))
        };
        // Apply one common limiter only when a reservoir would cross its energy floor.
        let limit = |value: f64, minimum: f64, rate: f64| {
            if rate < 0.0 { ((value - minimum) / (-rate * dt)).clamp(0.0, 1.0) }
            else { 1.0 }
        };
        let factor = limit(state.mass, self.mass_min, mass_rate)
            .min(limit(state.pulse, self.pulse_min, pulse_rate));
        let velocity = add(state.velocity, scale(work_k, factor * dt));
        let next = ParticleState {
            mass: (state.mass + mass_rate * factor * dt).max(self.mass_min),
            pulse: (state.pulse + pulse_rate * factor * dt).max(self.pulse_min),
            velocity,
            position: add(state.position, scale(add(state.velocity, velocity), dt * 0.5)),
        };
        if !next.mass.is_finite() || !next.pulse.is_finite() ||
            !next.velocity.is_finite() || !next.position.is_finite() {
            return Err(V2Error::NonFinite);
        }
        Ok(next)
    }
}

/// Diagnostic only. No conservation assertion or energy correction.
pub fn observed_energy(state: &ParticleState, c: &V2Constants) -> EnergyBreakdown {
    EnergyBreakdown {
        gravitational: c.k_g * state.mass * state.mass,
        levitational: c.k_l * state.pulse * state.pulse,
        kinetic: c.k_k * state.velocity.magnitude_squared(),
    }
}
