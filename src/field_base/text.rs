//! Text input field.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use unicode_width::UnicodeWidthStr;

use crate::field_base::{fill_row, render_label, BasicField};
use crate::style::FormStyle;
use crate::validation::Validator;

/// A single-line text input field.
pub struct TextInput {
    label: String,
    value: String,
    cursor_position: usize,
    selection: Option<(usize, usize)>,
    placeholder: Option<String>,
    required: bool,
    validators: Vec<Box<dyn Validator>>,
}

impl TextInput {
    /// Creates a new text input field.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            cursor_position: 0,
            selection: None,
            placeholder: None,
            required: false,
            validators: Vec::new(),
        }
    }

    /// Sets a placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Marks this field as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Adds a validator to this field.
    pub fn validator(mut self, validator: Box<dyn Validator>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Sets the initial value.
    pub fn initial_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_position = self.value.len();
        self
    }

    fn replace_selection(&mut self, replacement: &str) -> usize {
        let (start, end) = self.selection.take().expect("selection present");
        let end = end.min(self.value.len());
        let start = start.min(end);
        self.value.replace_range(start..end, replacement);
        start + replacement.len()
    }

    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection.take() {
            let end = end.min(self.value.len());
            let start = start.min(end);
            self.value.replace_range(start..end, "");
            self.cursor_position = start;
            true
        } else {
            false
        }
    }

    fn insert_char(&mut self, c: char) {
        if self.selection.is_some() {
            self.cursor_position = self.replace_selection(&c.to_string());
            return;
        }
        self.value.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    fn delete_char_before_cursor(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor_position > 0 {
            let prev_char_boundary = self.value[..self.cursor_position]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.value.remove(prev_char_boundary);
            self.cursor_position = prev_char_boundary;
        }
    }

    fn delete_char_at_cursor(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        self.selection = None;
        if self.cursor_position > 0 {
            self.cursor_position = self.value[..self.cursor_position]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    fn move_cursor_right(&mut self) {
        self.selection = None;
        if self.cursor_position < self.value.len() {
            self.cursor_position = self.value[self.cursor_position..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_position + i)
                .unwrap_or(self.value.len());
        }
    }

    fn move_cursor_home(&mut self) {
        self.selection = None;
        self.cursor_position = 0;
    }

    fn move_cursor_end(&mut self) {
        self.selection = None;
        self.cursor_position = self.value.len();
    }
}

impl BasicField for TextInput {
    fn value_str(&self) -> String {
        self.value.clone()
    }

    fn value_bool(&self) -> Option<bool> {
        None
    }

    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, style: &FormStyle) {
        if area.height < 1 || area.width < 1 {
            return;
        }

        // Render label and compute the input area
        let input_area = render_label(buf, area, &self.label, self.required, focused, style);
        let input_x = input_area.x;
        let input_width = input_area.width;

        if input_width == 0 {
            return;
        }

        // Determine what to display
        let (display_text, display_style) = if self.value.is_empty() {
            if let Some(ref placeholder) = self.placeholder {
                (placeholder.as_str(), style.placeholder)
            } else {
                ("", style.input)
            }
        } else {
            (self.value.as_str(), style.input)
        };

        // Render input value with background
        let input_bg_style = if focused {
            style.input_focused
        } else {
            style.input
        };

        // Fill input area with background
        fill_row(buf, input_area, input_bg_style);

        // Render the text
        let visible_text: String = display_text.chars().take(input_width as usize).collect();
        let selection = self.selection.filter(|_| !self.value.is_empty());
        let mut byte_idx = 0usize;
        for (i, c) in visible_text.chars().enumerate() {
            if input_x + i as u16 >= area.x + area.width {
                break;
            }
            let is_selected =
                selection.is_some_and(|(start, end)| byte_idx >= start && byte_idx < end);
            let char_style = if is_selected {
                display_style.add_modifier(Modifier::REVERSED)
            } else {
                display_style
            };
            buf[(input_x + i as u16, area.y)].set_char(c);
            buf[(input_x + i as u16, area.y)].set_style(char_style);
            byte_idx += c.len_utf8();
        }

        // Render cursor if focused
        if focused {
            let cursor_x = input_x + self.value[..self.cursor_position].width() as u16;
            if cursor_x < area.x + area.width {
                buf[(cursor_x, area.y)].set_style(
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Black)
                        .add_modifier(Modifier::SLOW_BLINK),
                );
            }
        }
    }

    fn handle_input(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'a' => self.move_cursor_home(),
                        'e' => self.move_cursor_end(),
                        'u' => {
                            self.value.clear();
                            self.cursor_position = 0;
                            self.selection = None;
                        }
                        _ => return false,
                    }
                } else {
                    self.insert_char(c);
                }
                true
            }
            KeyCode::Backspace => {
                self.delete_char_before_cursor();
                true
            }
            KeyCode::Delete => {
                self.delete_char_at_cursor();
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Home => {
                self.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.move_cursor_end();
                true
            }
            _ => false,
        }
    }

    fn on_focus(&mut self) {
        self.selection = Some((0, self.value.len()));
        self.cursor_position = self.value.len();
    }

    fn on_blur(&mut self) {
        self.selection = None;
    }

    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check required
        if self.required && self.value.trim().is_empty() {
            errors.push(format!("{} is required", self.label));
        }

        // Run validators
        for validator in &self.validators {
            if let Err(msg) = validator.validate(&self.value) {
                errors.push(msg);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
