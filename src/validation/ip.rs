use crate::Validator;

/// Validates that a value is a valid IPv4 address.
pub struct Ipv4;

impl Validator for Ipv4 {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Ok(()); // Empty is OK, use Required for that
        }

        if value.parse::<std::net::Ipv4Addr>().is_ok() {
            Ok(())
        } else {
            Err("Invalid IPv4 address".to_string())
        }
    }
}

/// Validates that a value is a valid IPv6 address.
pub struct Ipv6;

impl Validator for Ipv6 {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Ok(()); // Empty is OK, use Required for that
        }

        if value.parse::<std::net::Ipv6Addr>().is_ok() {
            Ok(())
        } else {
            Err("Invalid IPv6 address".to_string())
        }
    }
}
