//! Experimental computational core for Simplification Theory.
//!
//! No physical dynamics are implemented in this initial scaffold.

/// Identifies the version of this Rust implementation.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_has_a_version() {
        assert!(!CORE_VERSION.is_empty());
    }
}
