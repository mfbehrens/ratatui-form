use super::text::text_input;
use crate::base_field_types::TextInput;
use crate::field_types::{FieldAttributes, FieldType};
use crate::validation::numeric::Numeric;

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
