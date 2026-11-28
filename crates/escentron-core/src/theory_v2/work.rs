use super::{add, dot, norm, scale};
use crate::{EnergyBreakdown, FieldTotals, Vector3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkVectors {
    pub gravitational: Vector3,
    pub levitational: Vector3,
}

impl WorkVectors {
    pub(super) fn from_fields(fields: FieldTotals, e: EnergyBreakdown, floor: f64, f0: f64, dt: f64) -> Self {
        fn channel(f: Vector3, available: f64, f0: f64, dt: f64) -> Vector3 {
            let strength = norm(f);
            if strength == 0.0 || available <= 0.0 { return Vector3::ZERO; }
            let fraction = (dt * (strength / f0)).min(1.0);
            scale(scale(f, 1.0 / strength), available * fraction)
        }
        Self {
            gravitational: channel(fields.gravitational.0, e.gravitational - floor, f0, dt),
            levitational: channel(fields.levitational.0, e.levitational - floor, f0, dt),
        }
    }

    pub fn total(self) -> Vector3 {
        add(self.gravitational, self.levitational)
    }

    pub(super) fn exchange(self, delta: f64) -> (f64, f64) {
        let w = self.total();
        let length = norm(w);
        if length == 0.0 { return (0.0, 0.0); }
        let unit = scale(w, 1.0 / length);
        (dot(self.gravitational, unit) * (delta / length),
         dot(self.levitational, unit) * (delta / length))
    }
}
