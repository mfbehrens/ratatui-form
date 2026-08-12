//! Checkbox field.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

use crate::field::Field;
use crate::style::FormStyle;

/// A checkbox field.
pub struct Checkbox {
    label: String,
    checked: bool,
    required: bool,
}

impl Checkbox {
    /// Creates a new checkbox field.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            required: false,
        }
    }

    /// Sets the initial checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Marks this field as required (must be checked).
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

impl Field for Checkbox {
    fn label(&self) -> &str {
        &self.label
    }

    fn value_str(&self) -> String {
        if self.checked {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }

    fn value_bool(&self) -> Option<bool> {
        Some(self.checked)
    }

    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, style: &FormStyle) {
        if area.height < 1 || area.width < 4 {
            return;
        }

        let checkbox_style = if focused {
            style.input_focused
        } else {
            style.input
        };

        let label_style = if focused {
            style.label_focused
        } else {
            style.label
        };

        // Render checkbox
        let checkbox_char = if self.checked { "[✓]" } else { "[ ]" };
        for (i, c) in checkbox_char.chars().enumerate() {
            if area.x + (i as u16) < area.x + area.width {
                buf[(area.x + i as u16, area.y)].set_char(c);
                buf[(area.x + i as u16, area.y)].set_style(checkbox_style);
            }
        }

        // Render label
        let required_marker = if self.required { "*" } else { "" };
        let label_text = format!(" {}{}", self.label, required_marker);
        let label_x = area.x + 3;
        let remaining_width = area.width.saturating_sub(3);

        if remaining_width > 0 {
            let label_span = Span::styled(&label_text, label_style);
            let label_line = Line::from(label_span);
            let label_area = Rect {
                x: label_x,
                y: area.y,
                width: remaining_width.min(label_text.width() as u16),
                height: 1,
            };
            label_line.render(label_area, buf);
        }
    }

    fn handle_input(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle();
                true
            }
            _ => false,
        }
    }

    fn validate(&self) -> Result<(), Vec<String>> {
        if self.required && !self.checked {
            Err(vec![format!("{} must be checked", self.label)])
        } else {
            Ok(())
        }
    }

    fn is_required(&self) -> bool {
        self.required
    }
}
