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

impl FormValue for String {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
        let mut input = TextInput::new(spec.label);
        if let Some(placeholder) = spec.placeholder {
            input = input.placeholder(placeholder);
        }
        if spec.required {
            input = input.required();
        }
        for validator in spec.validators {
            input = input.validator(validator);
        }
        Box::new(input.initial_value(value.clone()))
    }

    fn form_extract<M: FormModel>(form: &Form<M>, index: usize) -> Result<Self, FormExtractError> {
        form.value_str(index).ok_or_else(|| FormExtractError {
            field_index: index,
            message: "field not found in form".to_string(),
        })
    }
}

impl FormValue for Option<String> {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
        let mut input = TextInput::new(spec.label);
        if let Some(placeholder) = spec.placeholder {
            input = input.placeholder(placeholder);
        }
        for validator in spec.validators {
            input = input.validator(validator);
        }
        Box::new(input.initial_value(value.clone().unwrap_or_default()))
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
                    let mut input = TextInput::new(spec.label);
                    if let Some(placeholder) = spec.placeholder {
                        input = input.placeholder(placeholder);
                    }
                    input = input.required().validator(Box::new(rules::Numeric));
                    for validator in spec.validators {
                        input = input.validator(validator);
                    }
                    Box::new(input.initial_value(value.to_string()))
                }

                fn form_extract<M: FormModel>(
                    form: &Form<M>,
                    index: usize,
                ) -> Result<Self, FormExtractError> {
                    let raw = form.value_str(index).ok_or_else(|| FormExtractError {
                        field_index: index,
                        message: "field not found in form".to_string(),
                    })?;
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

impl FormValue for std::net::Ipv4Addr {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
        let mut input = TextInput::new(spec.label);
        if let Some(placeholder) = spec.placeholder {
            input = input.placeholder(placeholder);
        }
        input = input.required().validator(Box::new(rules::Ipv4));
        for validator in spec.validators {
            input = input.validator(validator);
        }
        Box::new(input.initial_value(value.to_string()))
    }

    fn form_extract<M: FormModel>(form: &Form<M>, index: usize) -> Result<Self, FormExtractError> {
        let raw = form.value_str(index).ok_or_else(|| FormExtractError {
            field_index: index,
            message: "field not found in form".to_string(),
        })?;
        raw.parse::<Self>().map_err(|_| FormExtractError {
            field_index: index,
            message: "expected an IPv4 address".to_string(),
        })
    }
}

impl FormValue for std::net::Ipv6Addr {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
        let mut input = TextInput::new(spec.label);
        if let Some(placeholder) = spec.placeholder {
            input = input.placeholder(placeholder);
        }
        input = input.required().validator(Box::new(rules::Ipv6));
        for validator in spec.validators {
            input = input.validator(validator);
        }
        Box::new(input.initial_value(value.to_string()))
    }

    fn form_extract<M: FormModel>(form: &Form<M>, index: usize) -> Result<Self, FormExtractError> {
        let raw = form.value_str(index).ok_or_else(|| FormExtractError {
            field_index: index,
            message: "field not found in form".to_string(),
        })?;
        raw.parse::<Self>().map_err(|_| FormExtractError {
            field_index: index,
            message: "expected an IPv6 address".to_string(),
        })
    }
}
