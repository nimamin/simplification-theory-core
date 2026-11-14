//! Input validation only; not a physical simulation rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    NotFinite(&'static str),
    Negative(&'static str),
    NonPositive(&'static str),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFinite(name) => write!(f, "{name} must be finite"),
            Self::Negative(name) => write!(f, "{name} must be nonnegative"),
            Self::NonPositive(name) => write!(f, "{name} must be positive"),
        }
    }
}

impl std::error::Error for ValidationError {}

pub(crate) fn finite_nonnegative(value: f64, field: &'static str) -> Result<(), ValidationError> {
    if !value.is_finite() { return Err(ValidationError::NotFinite(field)); }
    if value < 0.0 { return Err(ValidationError::Negative(field)); }
    Ok(())
}

pub(crate) fn finite_positive(value: f64, field: &'static str) -> Result<(), ValidationError> {
    finite_nonnegative(value, field)?;
    if value == 0.0 { return Err(ValidationError::NonPositive(field)); }
    Ok(())
}
