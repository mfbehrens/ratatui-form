//! # ratatui-form
//!
//! Typed TUI forms built on [Ratatui]. Define a struct, derive
//! [`FormModel`], and get an interactive form whose edited values convert
//! back into the struct.//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//!
//! ## Features
//!
//! - **Typed forms** — `#[derive(FormModel)]` maps a struct to a form and back.
//! - **Fields** — [`TextInput`], [`Select`] (dropdown), [`Checkbox`].
//! - **Validation** — [`Required`], [`Email`], [`MinLength`], [`MaxLength`],
//!   [`Pattern`], [`Numeric`], or any `fn(&str) -> bool` / custom [`Validator`].
//! - **Theming** — [`FormStyle::dark`] / [`FormStyle::light`] presets, or
//!   override any component style.
//!
//! ## Quick start
//!
//! ```no_run
//! use ratatui_form::{FormModel, Form};
//!
//! #[derive(FormModel)]
//! #[form(title = "Sign Up")]
//! struct Signup {
//!     #[form(label = "Full Name", required, placeholder = "Ada Lovelace")]
//!     name: String,
//!
//!     #[form(label = "Age")]
//!     age: u8,
//!
//!     #[form(label = "Subscribe")]
//!     newsletter: bool,
//!
//!     #[form(skip)]
//!     id: u64,
//! }
//!
//! let model = Signup { name: "Ada".into(), age: 37, newsletter: true, id: 1 };
//! let form: Form<Signup> = model.into();
//! let back: Signup = Signup::try_from(form).unwrap();
//! # let _ = back;
//! ```
//!
//! See `examples/derive_form.rs` for the full event-loop wiring with
//! `crossterm` + `ratatui::Terminal`.
//!
//! ## Field attributes
//!
//! Each field of the struct becomes a form field, addressed by its position
//! in the struct:
//!
//! - `#[form(label = "…")]` — display label (defaults to the humanized name)
//! - `#[form(placeholder = "…")]` — text input placeholder
//! - `#[form(required)]` — the field must have a value to submit
//! - `#[form(validate = path)]` — a `fn(&str) -> bool` validator (repeatable)
//! - `#[form(skip)]` — exclude the field; restored via `Default`
//!
//! ## Custom field types
//!
//! The derive maps every field through the [`FormValue`] trait instead of
//! hardcoding a type→field mapping, so any type can be used in a form by
//! implementing [`FormValue`] for it. The library ships implementations for
//! `String`, `Option<String>`, `bool`, `std::net::Ipv4Addr`,
//! `std::net::Ipv6Addr`, and the numeric types (`u8`..`u64`, `i8`..`i64`,
//! `f32`, `f64`, and their size variants). See [`FormValue`] for an example
//! of a custom implementation.
//!
//! ## Validation
//!
//! Built-in validators live at the crate root. Implement [`Validator`] for custom
//! rules, or pass a plain `fn(&str) -> bool`:
//!
//! ```
//! use ratatui_form::Validator;
//!
//! struct Even;
//! impl Validator for Even {
//!     fn validate(&self, value: &str) -> Result<(), String> {
//!         match value.parse::<i32>() {
//!             Ok(n) if n % 2 == 0 => Ok(()),
//!             Ok(_) => Err("must be even".into()),
//!             Err(_) => Err("must be a number".into()),
//!         }
//!     }
//! }
//! ```
//!
//! ## Theming
//!
//! ```no_run
//! use ratatui_form::{FormModel, FormStyle, Form};
//! use ratatui::style::{Color, Modifier, Style};
//!
//! #[derive(FormModel)]
//! struct Settings {
//!     username: String,
//! }
//!
//! let style = FormStyle::new()
//!     .title(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
//!     .input_focused(Style::default().fg(Color::White).bg(Color::Blue));
//!
//! let model = Settings { username: "ada".into() };
//! let form: Form<Settings> = model.get_form().with_style(style);
//! # let _ = form;
//! ```

pub mod field;
pub mod form;
pub mod model;
pub mod navigation;
pub mod style;
pub mod validation;

mod form_value;

pub use field::{Checkbox, Field, Select, TextInput};
pub use form::FormResult;
pub use form_value::{FieldSpec, FormValue};
pub use model::{Form, FormExtractError, FormModel};
pub use navigation::FocusManager;
pub use ratatui_form_derive::FormModel;
pub use style::FormStyle;
pub use validation::rules::{Email, Ipv4, Ipv6, MaxLength, MinLength, Numeric, Pattern, Required};
pub use validation::{ValidationError, Validator};
