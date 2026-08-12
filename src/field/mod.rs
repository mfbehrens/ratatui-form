//! Field types for form inputs.

mod checkbox;
mod select;
mod text;

pub use checkbox::Checkbox;
pub use select::Select;
pub use text::TextInput;

use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::style::FormStyle;

/// Trait for form fields.
pub trait Field: Send + Sync {
    /// Returns the display label for this field.
    fn label(&self) -> &str;

    /// Returns the current value of the field as a string.
    fn value_str(&self) -> String;

    /// Returns the current value of the field as a boolean, if it is one.
    fn value_bool(&self) -> Option<bool>;

    /// Renders the field to the buffer.
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, style: &FormStyle);

    /// Handles keyboard input. Returns true if the input was consumed.
    fn handle_input(&mut self, event: &KeyEvent) -> bool;

    /// Validates the field and returns any error messages.
    fn validate(&self) -> Result<(), Vec<String>>;

    /// Returns the height needed to render this field.
    fn height(&self) -> u16 {
        1
    }

    /// Returns whether this field is required.
    fn is_required(&self) -> bool {
        false
    }
}
