//! Compile typed equation definitions to ordinary Rust functions during macro expansion.
use crate::Vector3 as Vec3;
type Scalar = f64;

pub(crate) fn dot(a: Vec3, b: Vec3) -> Scalar { a.dot(b) }
#[allow(dead_code)] // Available to equations even when the current laws do not use it.
pub(crate) fn cross(a: Vec3, b: Vec3) -> Vec3 { a.cross(b) }
pub(crate) fn length(v: Vec3) -> Scalar { v.length() }

macro_rules! equations {
    ($( $name:ident ($($arg:ident : $ty:ty),*) -> $out:ty = $value:expr; )*) => {
        $( #[inline] pub(crate) fn $name($($arg: $ty),*) -> $out { $value } )*
    };
}
include!("../theory/theory.eq");
