//! Three-dimensional position and velocity representation.
//!
//! This is a data type, not a choice of physical units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn magnitude_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn squared_speed_uses_all_components() {
        assert_eq!(Vector3::new(2.0, 3.0, 6.0).magnitude_squared(), 49.0);
    }
}
