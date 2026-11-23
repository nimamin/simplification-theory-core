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

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self::new(self.y * rhs.z - self.z * rhs.y,
                  self.z * rhs.x - self.x * rhs.z,
                  self.x * rhs.y - self.y * rhs.x)
    }

    pub fn length(self) -> f64 {
        self.x.hypot(self.y).hypot(self.z)
    }

    pub fn magnitude_squared(self) -> f64 {
        self.dot(self)
    }
}

impl std::ops::Add for Vector3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z) }
}

impl std::ops::Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z) }
}

impl std::ops::Mul<f64> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self { Self::new(self.x * rhs, self.y * rhs, self.z * rhs) }
}

impl std::ops::Div<f64> for Vector3 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

#[cfg(test)]
mod tests {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit/vector.rs"));
}
