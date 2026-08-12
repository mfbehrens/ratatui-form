# ratatui-form

[![crates.io](https://img.shields.io/crates/v/ratatui-form.svg)](https://crates.io/crates/ratatui-form)
[![docs.rs](https://docs.rs/ratatui-form/badge.svg)](https://docs.rs/ratatui-form)
[![CI](https://github.com/DavidLiedle/ratatui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/DavidLiedle/ratatui-form/actions/workflows/ci.yml)

Typed TUI forms built on [Ratatui](https://github.com/ratatui/ratatui). Define
a struct, derive `FormModel`, and get an interactive form whose edited values
convert back into the struct — no manual field wiring.

> **Note:** This crate was originally developed under the name `tform`, but was
> renamed to `ratatui-form` to avoid confusion with the unrelated
> [tform](https://crates.io/crates/tform) crate. If you were using the old
> name, please update your dependencies to `ratatui-form`.

## Features

- **Typed forms** - `#[derive(FormModel)]` maps a struct to a form and back
- **Pre-built Fields** - TextInput, Select (dropdown), Checkbox
- **Custom field types** - any type becomes a field via the `FormValue` trait
- **Validation** - Required, Email, MinLength, MaxLength, Pattern, Numeric, or
  any `fn(&str) -> bool` / custom `Validator`
- **Keyboard Navigation** - Tab, Shift+Tab, Arrow keys, Esc
- **Theming** - Customizable styles with dark/light presets

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ratatui-form = "0.1.1"
```

## Quick Start

```rust
use ratatui_form::{Form, FormModel};

#[derive(FormModel)]
#[form(title = "Sign Up")]
struct Signup {
    #[form(label = "Full Name", required, placeholder = "Ada Lovelace")]
    name: String,

    #[form(label = "Email", required, validate = is_valid_email)]
    email: String,

    #[form(label = "Age")]
    age: u8,

    #[form(label = "Subscribe")]
    newsletter: bool,

    #[form(skip)]
    id: u64,
}

// Validator functions are plain `fn(&str) -> bool`.
fn is_valid_email(value: &str) -> bool {
    value.contains('@') && value.contains('.')
}

// Build a form seeded with the model's current values...
let model = Signup {
    name: "Ada".into(),
    email: "ada@example.com".into(),
    age: 37,
    newsletter: true,
    id: 42,
};
let mut form = Form::<Signup>::from(model);

// ...and read the edited values back into the struct.
let edited: Signup = Signup::try_from(form)?;
```

Each struct field becomes a form field, addressed by its position in the
struct (skipped fields are excluded). `String` fields render as text inputs,
`bool` as checkboxes, and numeric / IP-address types as validated text inputs.

## Field Attributes

- `#[form(label = "…")]` — display label (defaults to the humanized field name)
- `#[form(placeholder = "…")]` — text input placeholder
- `#[form(required)]` — the field must have a value to submit
- `#[form(validate = path)]` — a `fn(&str) -> bool` validator (repeatable)
- `#[form(skip)]` — exclude the field from the form; restored via `Default`

Struct attribute:

- `#[form(title = "…")]` — the form title (defaults to the struct name)

## Custom Field Types

The derive maps every field through the `FormValue` trait instead of
hardcoding a type→field mapping, so any type can be used in a form by
implementing `FormValue`. The library ships implementations for `String`,
`Option<String>`, `bool`, `std::net::Ipv4Addr`, `std::net::Ipv6Addr`, and the
numeric types (`u8`..`u64`, `i8`..`i64`, `f32`, `f64`, and their size
variants).

```rust
use ratatui_form::{Field, FieldSpec, Form, FormExtractError, FormModel, FormValue, TextInput};

#[derive(Clone, Debug, PartialEq)]
struct Port(u16);

impl FormValue for Port {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn Field> {
        let mut input = TextInput::new(spec.label);
        if spec.required {
            input = input.required();
        }
        Box::new(input.initial_value(value.0.to_string()))
    }

    fn form_extract<M: FormModel>(form: &Form<M>, index: usize) -> Result<Self, FormExtractError> {
        let raw = form.value_str(index).ok_or_else(|| FormExtractError {
            field_index: index,
            message: "field not found in form".to_string(),
        })?;
        let port = raw.parse::<u16>().map_err(|_| FormExtractError {
            field_index: index,
            message: "expected a port number (0-65535)".to_string(),
        })?;
        Ok(Port(port))
    }
}

#[derive(Debug, FormModel)]
struct ServerConfig {
    #[form(label = "Port", required)]
    port: Port,
}
```

## Validation

### Built-in Validators

```rust
use ratatui_form::{Required, Email, MinLength, MaxLength, Pattern, Numeric};

// Required - field cannot be empty
.validator(Box::new(Required))

// Email - valid email format
.validator(Box::new(Email))

// MinLength - minimum character count
.validator(Box::new(MinLength(3)))

// MaxLength - maximum character count
.validator(Box::new(MaxLength(100)))

// Numeric - field must parse as a number
.validator(Box::new(Numeric))

// Pattern - custom regex
.validator(Box::new(Pattern::new(r"^\d{3}-\d{4}$", "Invalid format")))

// Pre-built patterns
.validator(Box::new(Pattern::zip_code()))   // US ZIP code
.validator(Box::new(Pattern::phone()))      // US phone number
.validator(Box::new(Pattern::date()))       // YYYY-MM-DD
```

### Custom Validators

Implement the `Validator` trait:

```rust
use ratatui_form::Validator;

struct EvenNumber;

impl Validator for EvenNumber {
    fn validate(&self, value: &str) -> Result<(), String> {
        match value.parse::<i32>() {
            Ok(n) if n % 2 == 0 => Ok(()),
            Ok(_) => Err("Must be an even number".to_string()),
            Err(_) => Err("Must be a number".to_string()),
        }
    }
}
```

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Up` / `Down` | Navigate fields (or dropdown options when open) |
| `Enter` | Submit form (on button) / Select option (in dropdown) |
| `Space` | Toggle checkbox / Open dropdown |
| `Esc` | Cancel form / Close dropdown |
| `Left` / `Right` | Move cursor in text fields |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `Ctrl+A` | Move cursor to start |
| `Ctrl+E` | Move cursor to end |
| `Ctrl+U` | Clear field |

When a text field gains focus its value is selected, so the first keystroke
replaces it. Press `End` or an arrow key to clear the selection and append
instead.

## Theming

### Using Presets

```rust
use ratatui_form::FormStyle;

// Dark theme (default)
FormStyle::dark()

// Light theme
FormStyle::light()
```

### Custom Styles

```rust
use ratatui_form::FormStyle;
use ratatui::style::{Color, Modifier, Style};

let custom_style = FormStyle::new()
    .title(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
    .label(Style::default().fg(Color::White))
    .label_focused(Style::default().fg(Color::Yellow))
    .input(Style::default().fg(Color::White).bg(Color::DarkGray))
    .input_focused(Style::default().fg(Color::White).bg(Color::Blue))
    .error(Style::default().fg(Color::Red))
    .button(Style::default().fg(Color::White).bg(Color::DarkGray))
    .button_focused(Style::default().fg(Color::Black).bg(Color::Green));

let form = model.get_form().with_style(custom_style);
```

## Examples

```bash
cargo run --example derive_form   # typed form from a struct
cargo run --example custom_field  # completely custom field types via FormValue
```

## License

MIT
