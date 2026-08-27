use crate::Validator;

/// Validates that a value parses as a number.
pub struct Numeric;

impl Validator for Numeric {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Ok(()); // Empty is OK, use Required for that
        }

        if value.parse::<f64>().is_ok() {
            Ok(())
        } else {
            Err("Must be a number".to_string())
        }
    }
}
