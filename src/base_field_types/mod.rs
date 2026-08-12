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
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use crate::style::FormStyle;

/// Renders a field label ("<label>[required]: ") and returns the area left
/// over for the input widget itself.
pub(crate) fn render_label(
    buf: &mut Buffer,
    area: Rect,
    label: &str,
    required: bool,
    focused: bool,
    style: &FormStyle,
) -> Rect {
    let label_style = if focused {
        style.label_focused
    } else {
        style.label
    };
    let required_marker = if required { "*" } else { "" };
    let label_text = format!("{label}{required_marker}: ");
    let label_width = label_text.width().min(area.width as usize);

    Line::from(Span::styled(&label_text, label_style)).render(
        Rect {
            x: area.x,
            y: area.y,
            width: label_width as u16,
            height: 1,
        },
        buf,
    );

    Rect {
        x: area.x + label_width as u16,
        y: area.y,
        width: area.width.saturating_sub(label_width as u16),
        height: area.height,
    }
}

/// Fills a row with a background style.
pub(crate) fn fill_row(buf: &mut Buffer, area: Rect, style: ratatui::style::Style) {
    for x in area.x..area.x + area.width {
        buf[(x, area.y)].set_style(style);
        buf[(x, area.y)].set_char(' ');
    }
}

/// Trait for form fields.
pub trait BasicFieldType: Send + Sync {
    // type ValueType;
    // /// Returns the current value of the field as a string.
    // fn value(&self) -> Self::ValueType;
    // /// Returns the current value of the field as a string.
    fn value_str(&self) -> String;

    /// Returns the current value of the field as a boolean, if it is one.
    fn value_bool(&self) -> Option<bool>;

    /// Renders the field to the buffer.
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, style: &FormStyle);

    /// Handles keyboard input. Returns true if the input was consumed.
    fn handle_input(&mut self, event: &KeyEvent) -> bool;

    /// Called when the field gains focus.
    fn on_focus(&mut self) {}

    /// Called when the field loses focus.
    fn on_blur(&mut self) {}

    /// Validates the field and returns any error messages.
    fn validate(&self) -> Result<(), Vec<String>>;

    /// Returns the height needed to render this field.
    fn height(&self) -> u16 {
        1
    }
}
