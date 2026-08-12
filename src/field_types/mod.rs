use crate::validation::Validator;
use crate::BasicFieldType;

/// Configuration for building a form field from a model value.
pub struct FieldAttributes {
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
/// use ratatui_form::{FieldAttributes, FieldType, TextInput};
///
/// #[derive(Clone)]
/// struct Port(u16);
///
/// impl FieldType for Port {
///     type BaseFieldType = TextInput;
///
///     fn form_field(spec: FieldAttributes, value: &Self) -> Self::BaseFieldType {
///         TextInput::new(spec.label).initial_value(value.0.to_string())
///     }
///
///     fn form_extract(field: &Self::BaseFieldType) -> Result<Self, String> {
///         let raw = field.value();
///         let port = raw.parse().map_err(|_| "expected a port number".to_string())?;
///         Ok(Port(port))
///     }
/// }
/// ```
pub trait FieldType: Sized {
    /// Type of the Form Field widget (e.g. `TextInput`, `Checkbox`, `Select`).
    type BaseFieldType: BasicFieldType;

    /// Builds a form field widget seeded with `value`.
    fn form_field(spec: FieldAttributes, value: &Self) -> Self::BaseFieldType;

    /// Extracts a value of this type from the field widget.
    fn form_extract(field: &Self::BaseFieldType) -> Result<Self, String>;
}

mod bool;
mod ip;
mod numeric;
mod text;
