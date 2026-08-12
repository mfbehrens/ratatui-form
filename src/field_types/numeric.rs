use super::text::text_input;
use crate::base_field_types::TextInput;
use crate::field_types::{FieldAttributes, FieldType};
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

macro_rules! impl_numeric_form_value {
    ($($ty:ty),* $(,)?) => {
        $(
            impl FieldType for $ty {
                type BaseFieldType = TextInput;
                fn form_field(spec: FieldAttributes, value: &Self) -> Self::BaseFieldType {
                    text_input(spec, value.to_string(), Some(Box::new(Numeric))).required()
                }

                fn form_extract(field: &Self::BaseFieldType) -> Result<Self, String> {
                    field.value().parse::<Self>().map_err(|_| "expected a number".to_string())
                }
            }
        )*
    };
}

impl_numeric_form_value!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64,);
