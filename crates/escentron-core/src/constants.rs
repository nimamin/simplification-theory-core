//! Explicit parameters of a simulation; no arbitrary default values.
use crate::validation::{finite_nonnegative, finite_positive, ValidationError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelConstants {
    /// G: common field scale parameter.
    pub g: f64,
    /// E_g = k_g * m
    pub k_g: f64,
    /// E_l = k_l * p
    pub k_l: f64,
    /// E_k = k_k * |v|^2
    pub k_k: f64,
    /// Shared per-particle cosmic energy E_c.
    pub cosmic_energy: f64,
}

impl ModelConstants {
    pub fn validate(&self) -> Result<(), ValidationError> {
        finite_nonnegative(self.g, "G")?;
        finite_nonnegative(self.k_g, "k_g")?;
        finite_nonnegative(self.k_l, "k_l")?;
        finite_nonnegative(self.k_k, "k_k")?;
        finite_positive(self.cosmic_energy, "E_c")?;
        Ok(())
    }
}
