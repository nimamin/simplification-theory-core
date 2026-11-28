use super::{add, norm, scale, V2Constants, V2Error};
use crate::{FieldTotals, GravitationalField, LevitationalField, ParticleState, Vector3};

pub fn incident_fields(
    states: &[ParticleState],
    target: usize,
    c: &V2Constants,
) -> Result<FieldTotals, V2Error> {
    c.validate()?;
    let receiver = states.get(target).ok_or(V2Error::InvalidInput)?;
    receiver.validate().map_err(|_| V2Error::InvalidInput)?;
    let mut gravity = Vector3::ZERO;
    let mut levitation = Vector3::ZERO;
    for (i, source) in states.iter().enumerate() {
        if i == target { continue; }
        source.validate().map_err(|_| V2Error::InvalidInput)?;
        let d = add(receiver.position, scale(source.position, -1.0));
        let r = norm(d);
        if r == 0.0 || !r.is_finite() { return Err(V2Error::SingularField); }
        let unit = scale(d, 1.0 / r);
        gravity = add(gravity, scale(unit, -c.c_g * source.mass / r / r));
        levitation = add(levitation, scale(unit, c.c_l * source.pulse / r / r / r));
    }
    if !gravity.is_finite() || !levitation.is_finite() { return Err(V2Error::NonFinite); }
    Ok(FieldTotals {
        gravitational: GravitationalField(gravity),
        levitational: LevitationalField(levitation),
    })
}
