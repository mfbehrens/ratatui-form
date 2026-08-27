use crate::base_field_types::TextInput;
use crate::field_types::{FieldAttributes, FieldType};
use crate::validation::Validator;

/// Builds a text input from a spec, seeded with `value`.
///
/// `rule` is an optional type-specific validator (e.g. `Numeric`); it is
/// applied before any validators supplied in the spec.
pub(super) fn text_input(
    spec: FieldAttributes,
    value: String,
    rule: Option<Box<dyn Validator>>,
) -> TextInput {
    let mut input = TextInput::new(spec.label);
    if let Some(placeholder) = spec.placeholder {
        input = input.placeholder(placeholder);
    }
    if spec.required {
        input = input.required();
    }
    if let Some(rule) = rule {
        input = input.validator(rule);
    }
    for validator in spec.validators {
        input = input.validator(validator);
    }
    input.initial_value(value)
}

impl FieldType for String {
    type BaseFieldType = TextInput;
    fn form_field(spec: FieldAttributes, value: &Self) -> Self::BaseFieldType {
        text_input(spec, value.clone(), None)
    }

    fn form_extract(field: &Self::BaseFieldType) -> Result<Self, String> {
        Ok(field.value().to_string())
    }
}

impl FieldType for Option<String> {
    type BaseFieldType = TextInput;
    fn form_field(spec: FieldAttributes, value: &Self) -> Self::BaseFieldType {
        text_input(spec, value.clone().unwrap_or_default(), None)
    }

    fn form_extract(field: &Self::BaseFieldType) -> Result<Self, String> {
        let val = field.value();
        if val.is_empty() {
            Ok(None)
        } else {
            Ok(Some(val.to_string()))
        }
    }
}
