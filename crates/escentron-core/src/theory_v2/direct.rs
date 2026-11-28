//! Experimental direct integration of mass, pulse and velocity.
use super::{add, dot, norm, scale, V2Constants, V2Error};
use crate::{EnergyBreakdown, FieldTotals, ParticleState};

#[derive(Debug, Clone, Copy)]
pub struct DirectModel {
    pub constants: V2Constants,
    pub initial_cosmic_energy: f64,
    pub mass_max: f64,
    pub pulse_max: f64,
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
        })
    }

    pub fn advance(&self, state: &ParticleState, fields: FieldTotals, dt: f64)
        -> Result<ParticleState, V2Error> {
        if !dt.is_finite() || dt <= 0.0 { return Err(V2Error::InvalidInput); }
        let c = self.constants;
        let g = scale(fields.gravitational.0, c.k_w * (state.mass / self.mass_max) / c.reference_field);
        let l = scale(fields.levitational.0, c.k_w * (state.pulse / self.pulse_max) / c.reference_field);
        let dv = scale(add(g, l), dt);
        let velocity = add(state.velocity, dv);
        // Experimental signed coordinate exchange: allocate the speed
        // change by projection onto the resultant acceleration.
        let a = add(g, l);
        let length = norm(a);
        let ds = norm(velocity) - norm(state.velocity);
        let (dm, dp) = if length == 0.0 { (0.0, 0.0) } else {
            let unit = scale(a, 1.0 / length);
            (-dot(g, unit) / length * ds, -dot(l, unit) / length * ds)
        };
        let next = ParticleState {
            mass: state.mass + dm,
            pulse: state.pulse + dp,
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

/// Diagnostic only. No floor, conservation assertion or energy correction.
pub fn observed_energy(state: &ParticleState, c: &V2Constants) -> EnergyBreakdown {
    EnergyBreakdown {
        gravitational: c.k_g * state.mass * state.mass,
        levitational: c.k_l * state.pulse * state.pulse,
        kinetic: c.k_k * state.velocity.magnitude_squared(),
    }
}
