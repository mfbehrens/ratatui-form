//! Mapping between model values and form fields.

use crate::field_base::{Checkbox, TextInput};
use crate::validation::{rules, Validator};

/// Configuration for building a form field from a model value.
pub struct FieldSpec {
    /// Display label of the field.
    pub label: String,
    /// Optional placeholder text.
    pub placeholder: Option<String>,
    /// Whether the field is required.
    pub required: bool,
    /// Validators applied to the field's string value.
    pub validators: Vec<Box<dyn Validator>>,
}

/// A value type that can be rendered in a form and read back from it.
///
/// `#[derive(FormModel)]` delegates field construction and extraction to this
/// trait, so any type with a `FormValue` implementation can be used as a form
/// field. The library provides implementations for `String`, `Option<String>`,
/// `bool`, `std::net::Ipv4Addr`, `std::net::Ipv6Addr`, and the numeric types.
///
/// Implement this trait to add completely custom field types:
///
/// ```
/// use ratatui_form::{FieldSpec, FormValue, TextInput};
///
/// #[derive(Clone)]
/// struct Port(u16);
///
/// impl FormValue for Port {
///     type FieldType = TextInput;
///
///     fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType {
///         TextInput::new(spec.label).initial_value(value.0.to_string())
///     }
///
///     fn form_extract(field: &Self::FieldType) -> Result<Self, String> {
///         let raw = field.value();
///         let port = raw.parse().map_err(|_| "expected a port number".to_string())?;
///         Ok(Port(port))
///     }
/// }
/// ```
pub trait FormValue: Sized {
    /// Type of the Form Field widget (e.g. `TextInput`, `Checkbox`, `Select`).
    type FieldType;

    /// Builds a form field widget seeded with `value`.
    fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType;

    /// Extracts a value of this type from the field widget.
    fn form_extract(field: &Self::FieldType) -> Result<Self, String>;
}

/// Builds a text input from a spec, seeded with `value`.
///
/// `rule` is an optional type-specific validator (e.g. `Numeric`); it is
/// applied before any validators supplied in the spec.
fn text_input(spec: FieldSpec, value: String, rule: Option<Box<dyn Validator>>) -> TextInput {
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

impl FormValue for String {
    type FieldType = TextInput;
    fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType {
        text_input(spec, value.clone(), None)
    }

    fn form_extract(field: &Self::FieldType) -> Result<Self, String> {
        Ok(field.value().to_string())
    }
}

impl FormValue for Option<String> {
    type FieldType = TextInput;
    fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType {
        text_input(spec, value.clone().unwrap_or_default(), None)
    }

    fn form_extract(field: &Self::FieldType) -> Result<Self, String> {
        let val = field.value();
        if val.is_empty() {
            Ok(None)
        } else {
            Ok(Some(val.to_string()))
        }
    }
}

impl FormValue for bool {
    type FieldType = Checkbox;
    fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType {
        let mut checkbox = Checkbox::new(spec.label);
        if spec.required {
            checkbox = checkbox.required();
        }
        checkbox.checked(*value)
    }

    fn form_extract(field: &Self::FieldType) -> Result<Self, String> {
        Ok(field.is_checked())
    }
}

macro_rules! impl_numeric_form_value {
    ($($ty:ty),* $(,)?) => {
        $(
            impl FormValue for $ty {
                type FieldType = TextInput;
                fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType {
                    text_input(spec, value.to_string(), Some(Box::new(rules::Numeric))).required()
                }

                fn form_extract(field: &Self::FieldType) -> Result<Self, String> {
                    field.value().parse::<Self>().map_err(|_| "expected a number".to_string())
                }
            }
        )*
    };
}

impl_numeric_form_value!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64,);

macro_rules! impl_ip_form_value {
    ($($ty:ty => $rule:expr, $message:literal;)*) => {
        $(
            impl FormValue for $ty {
                type FieldType = TextInput;
                fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType {
                    text_input(spec, value.to_string(), Some(Box::new($rule))).required()
                }

                fn form_extract(field: &Self::FieldType) -> Result<Self, String> {
                    field.value().parse::<Self>().map_err(|_| $message.to_string())
                }
            }
        )*
    };
}

impl_ip_form_value!(
    std::net::Ipv4Addr => rules::Ipv4, "expected an IPv4 address";
    std::net::Ipv6Addr => rules::Ipv6, "expected an IPv6 address";
);
