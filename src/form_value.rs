//! Mapping between model values and form fields.

use crate::field::{Checkbox, Field, TextInput};
use crate::model::{Form, FormExtractError, FormModel};
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
/// use ratatui_form::{FieldSpec, Form, FormExtractError, FormModel, FormValue, TextInput};
///
/// #[derive(Clone)]
/// struct Port(u16);
///
/// impl FormValue for Port {
///     fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn ratatui_form::Field> {
///         Box::new(TextInput::new(spec.label).initial_value(value.0.to_string()))
///     }
///
///     fn form_extract<M: FormModel>(
///         form: &Form<M>,
///         index: usize,
///     ) -> Result<Self, FormExtractError> {
///         let value = form.value_str(index).ok_or_else(|| FormExtractError {
///             field_index: index,
///             message: "field not found in form".to_string(),
///         })?;
///         let port = value.parse().map_err(|_| FormExtractError {
///             field_index: index,
///             message: "expected a port number".to_string(),
///         })?;
///         Ok(Port(port))
///     }
/// }
/// ```
pub trait FormValue: Sized {
    /// Builds a form field widget seeded with `value`.
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field>;

    /// Extracts a value of this type from the form at `index`.
    fn form_extract<M: FormModel>(form: &Form<M>, index: usize) -> Result<Self, FormExtractError>;
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

/// The error returned when a field index is out of bounds.
fn missing_field(index: usize) -> FormExtractError {
    FormExtractError {
        field_index: index,
        message: "field not found in form".to_string(),
    }
}

impl FormValue for String {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
        Box::new(text_input(spec, value.clone(), None))
    }

    fn form_extract<M: FormModel>(form: &Form<M>, index: usize) -> Result<Self, FormExtractError> {
        form.value_str(index).ok_or_else(|| missing_field(index))
    }
}

impl FormValue for Option<String> {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
        Box::new(text_input(spec, value.clone().unwrap_or_default(), None))
    }

    fn form_extract<M: FormModel>(form: &Form<M>, index: usize) -> Result<Self, FormExtractError> {
        Ok(match form.value_str(index) {
            Some(value) if value.is_empty() => None,
            Some(value) => Some(value),
            None => None,
        })
    }
}

impl FormValue for bool {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
        let mut checkbox = Checkbox::new(spec.label);
        if spec.required {
            checkbox = checkbox.required();
        }
        Box::new(checkbox.checked(*value))
    }

    fn form_extract<M: FormModel>(form: &Form<M>, index: usize) -> Result<Self, FormExtractError> {
        form.value_bool(index).ok_or_else(|| FormExtractError {
            field_index: index,
            message: "expected a boolean".to_string(),
        })
    }
}

macro_rules! impl_numeric_form_value {
    ($($ty:ty),* $(,)?) => {
        $(
            impl FormValue for $ty {
                fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
                    // Numeric fields have no meaningful empty value, so they
                    // are always required regardless of `spec.required`.
                    Box::new(text_input(spec, value.to_string(), Some(Box::new(rules::Numeric))).required())
                }

                fn form_extract<M: FormModel>(
                    form: &Form<M>,
                    index: usize,
                ) -> Result<Self, FormExtractError> {
                    let raw = form.value_str(index).ok_or_else(|| missing_field(index))?;
                    raw.parse::<Self>().map_err(|_| FormExtractError {
                        field_index: index,
                        message: "expected a number".to_string(),
                    })
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
                fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
                    // IP fields have no meaningful empty value, so they are
                    // always required regardless of `spec.required`.
                    Box::new(text_input(spec, value.to_string(), Some(Box::new($rule))).required())
                }

                fn form_extract<M: FormModel>(
                    form: &Form<M>,
                    index: usize,
                ) -> Result<Self, FormExtractError> {
                    let raw = form.value_str(index).ok_or_else(|| missing_field(index))?;
                    raw.parse::<Self>().map_err(|_| FormExtractError {
                        field_index: index,
                        message: $message.to_string(),
                    })
                }
            }
        )*
    };
}

impl_ip_form_value!(
    std::net::Ipv4Addr => rules::Ipv4, "expected an IPv4 address";
    std::net::Ipv6Addr => rules::Ipv6, "expected an IPv6 address";
);
