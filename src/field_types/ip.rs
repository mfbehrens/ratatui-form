use super::text::text_input;
use crate::base_field_types::TextInput;
use crate::field_types::{FieldAttributes, FieldType};
use crate::validation::Validator;

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

macro_rules! impl_ip_form_value {
    ($($ty:ty => $rule:expr, $message:literal;)*) => {
        $(
            impl FieldType for $ty {
                type BaseFieldType = TextInput;
                fn form_field(spec: FieldAttributes, value: &Self) -> Self::BaseFieldType {
                    text_input(spec, value.to_string(), Some(Box::new($rule))).required()
                }

                fn form_extract(field: &Self::BaseFieldType) -> Result<Self, String> {
                    field.value().parse::<Self>().map_err(|_| $message.to_string())
                }
            }
        )*
    };
}

impl_ip_form_value!(
    std::net::Ipv4Addr => Ipv4, "expected an IPv4 address";
    std::net::Ipv6Addr => Ipv6, "expected an IPv6 address";
);
