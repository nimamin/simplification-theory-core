//! Reference implementation of the three postulates, version 2.
mod evolution;
mod fields;
mod work;

pub use evolution::{advance, advance_system, energies, V2Constants, V2Error};
pub use fields::incident_fields;
pub use work::WorkVectors;

pub const THEORY_VERSION: &str = "2.0";
use crate::Vector3;

fn add(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}
fn scale(a: Vector3, s: f64) -> Vector3 {
    Vector3::new(a.x * s, a.y * s, a.z * s)
}
fn dot(a: Vector3, b: Vector3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}
fn norm(a: Vector3) -> f64 {
    a.x.hypot(a.y).hypot(a.z)
}
