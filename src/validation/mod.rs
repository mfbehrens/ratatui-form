//! Validation traits and types.

pub mod rules;

/// A validation error for a specific field.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The index of the field that failed validation.
    pub field_index: usize,
    /// The error message.
    pub message: String,
}

/// Trait for field validators.
pub trait Validator: Send + Sync {
    /// Validates a value and returns an error message if invalid.
    fn validate(&self, value: &str) -> Result<(), String>;
}

impl Validator for fn(&str) -> bool {
    fn validate(&self, value: &str) -> Result<(), String> {
        if self(value) {
            Ok(())
        } else {
            Err("invalid value".to_string())
        }
    }
}
