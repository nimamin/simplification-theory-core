use super::{add, dot, norm, scale};
use crate::{EnergyBreakdown, FieldTotals, Vector3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkVectors {
    pub gravitational: Vector3,
    pub levitational: Vector3,
}

impl WorkVectors {
    pub(super) fn from_fields(fields: FieldTotals, e: EnergyBreakdown, f0: f64, dt: f64) -> Self {
        Self {
            gravitational: scale(fields.gravitational.0, e.gravitational * (dt / f0)),
            levitational: scale(fields.levitational.0, e.levitational * (dt / f0)),
        }
    }

    pub fn total(self) -> Vector3 {
        add(self.gravitational, self.levitational)
    }

    fn scaled(self, factor: f64) -> Self {
        Self {
            gravitational: scale(self.gravitational, factor),
            levitational: scale(self.levitational, factor),
        }
    }

    fn exchange(self, delta: f64) -> (f64, f64) {
        let w = self.total();
        let length = norm(w);
        if length == 0.0 { return (0.0, 0.0); }
        let unit = scale(w, 1.0 / length);
        (dot(self.gravitational, unit) * (delta / length),
         dot(self.levitational, unit) * (delta / length))
    }

    fn transfers(self, j: Vector3) -> (f64, f64) {
        let w = self.total();
        let next = add(j, w);
        let denominator = norm(next) + norm(j);
        let delta = if denominator > 0.0 {
            dot(w, add(next, j)) / denominator
        } else { 0.0 };
        self.exchange(delta)
    }

    /// Limit actual energy delivered, not a reservoir's ability to receive.
    /// A common scale preserves the raw resultant direction and signed shares.
    pub(super) fn limited(self, j: Vector3, e: EnergyBreakdown, floor: f64) -> (Self, f64, f64) {
        let fits = |(qg, ql): (f64, f64)| {
            qg <= (e.gravitational - floor).max(0.0)
                && ql <= (e.levitational - floor).max(0.0)
        };
        let raw = self.transfers(j);
        if fits(raw) { return (self, raw.0, raw.1); }
        let mut low = 0.0;
        let mut high = 1.0;
        for _ in 0..64 {
            let mid = (low + high) * 0.5;
            if fits(self.scaled(mid).transfers(j)) { low = mid; }
            else { high = mid; }
        }
        let work = self.scaled(low);
        let (qg, ql) = work.transfers(j);
        (work, qg, ql)
    }
}
