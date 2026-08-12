//! Typed form support: derive a form from a model struct.

pub use crate::form::{Form, FormFields};

/// An error produced while extracting a model from a [`Form`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormExtractError {
    /// The index of the field that failed.
    pub field_index: usize,
    /// The name of the field that failed (if available).
    pub field_name: String,
    /// A description of the failure.
    pub message: String,
}

impl FormExtractError {
    /// Creates a new extraction error.
    pub fn new(field_index: usize, field_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field_index,
            field_name: field_name.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for FormExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.field_name.is_empty() {
            write!(f, "field {}: {}", self.field_index, self.message)
        } else {
            write!(f, "field {} ({}): {}", self.field_index, self.field_name, self.message)
        }
    }
}

impl std::error::Error for FormExtractError {}

/// A model type that can be rendered as a [`Form`].
///
/// This trait is implemented by the `#[derive(FormModel)]` macro. The derive
/// generates a submodule containing a custom `Fields` struct that holds
/// typed form input controls, and implements `FormModel` and `TryFrom<Form<Fields>>`.
pub trait FormModel: Sized {
    /// The strongly-typed fields container for this model.
    type Fields: FormFields;

    /// Returns a [`Form`] seeded with this struct's current values.
    fn get_form(&self) -> Form<Self::Fields>;
}

/// Helper type alias for a Form for model `T`.
pub type FormFor<T> = Form<<T as FormModel>::Fields>;

impl<T: FormModel> From<T> for Form<T::Fields> {
    fn from(model: T) -> Self {
        model.get_form()
    }
}
