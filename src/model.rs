//! Typed form support: derive a form from a model struct.

use std::marker::PhantomData;

use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::field::Field;
use crate::form::{FormEngine, FormResult};
use crate::style::FormStyle;
use crate::validation::ValidationError;

/// An error produced while extracting a model from a [`Form`].
#[derive(Debug, Clone)]
pub struct FormExtractError {
    /// The index of the field that failed.
    pub field_index: usize,
    /// A description of the failure.
    pub message: String,
}

impl std::fmt::Display for FormExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "field {}: {}", self.field_index, self.message)
    }
}

impl std::error::Error for FormExtractError {}

/// A model type that can be rendered as a [`Form`].
///
/// This trait is implemented by the `#[derive(FormModel)]` macro. The derive
/// generates a `get_form` method that builds a form seeded with the current
/// values of the struct, and a `TryFrom<Form<Self>>` impl that converts
/// the edited form back into a new struct instance.
pub trait FormModel: Sized {
    /// Returns a [`Form`] seeded with this struct's current values.
    fn get_form(&self) -> Form<Self>;
}

/// A form tied to a model type `T`.
///
/// `Form<T>` wraps a [`FormEngine`] and carries the model type, so the edited
/// values can be converted back into a `T` with `T::try_from(form)`.
pub struct Form<T: FormModel> {
    inner: FormEngine,
    marker: PhantomData<T>,
}

impl<T: FormModel> From<T> for Form<T> {
    fn from(model: T) -> Self {
        model.get_form()
    }
}

impl<T: FormModel> Form<T> {
    /// Creates a new, empty typed form.
    ///
    /// This is used by the `FormModel` derive and is not intended to be called
    /// directly; prefer `model.into()` or `model.get_form()`.
    #[doc(hidden)]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            inner: FormEngine::new(title),
            marker: PhantomData,
        }
    }

    /// Appends a field to the form.
    ///
    /// This is used by the `FormModel` derive and is not intended to be called
    /// directly.
    #[doc(hidden)]
    pub fn push(&mut self, field: Box<dyn Field>) {
        self.inner.push(field);
    }

    /// Applies a style to the form.
    pub fn with_style(mut self, style: FormStyle) -> Self {
        self.inner.set_style(style);
        self
    }

    /// Returns the raw string value of the field at `index`, if any.
    pub fn value_str(&self, index: usize) -> Option<String> {
        self.inner.value_str(index)
    }

    /// Returns the boolean value of the field at `index`, if it is one.
    pub fn value_bool(&self, index: usize) -> Option<bool> {
        self.inner.value_bool(index)
    }

    /// Returns the current form result.
    pub fn result(&self) -> &FormResult {
        self.inner.result()
    }

    /// Returns whether the form is still active.
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    /// Returns validation errors from the last submit attempt.
    pub fn validation_errors(&self) -> &[ValidationError] {
        self.inner.validation_errors()
    }

    /// Handles keyboard input.
    pub fn handle_input(&mut self, event: KeyEvent) {
        self.inner.handle_input(event);
    }

    /// Renders the form to a buffer.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        self.inner.render(area, buf);
    }
}
