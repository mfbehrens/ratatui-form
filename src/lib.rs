//! # ratatui-form
//!
//! Typed TUI forms built on [Ratatui]. Define a struct, derive
//! [`FormModel`], and get an interactive form whose edited values convert
//! back into the struct.
//!
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
//! use ratatui_form::{FormModel, FormFor};
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
//! let mut form: FormFor<Signup> = model.into();
//! // Access typed fields directly:
//! assert_eq!(form.fields.name.value(), "Ada");
//! let back: Signup = Signup::try_from(form).unwrap();
//! # let _ = back;
//! ```
//!
//! See `examples/derive_form.rs` for the full event-loop wiring with
//! `crossterm` + `ratatui::Terminal`.

pub mod field_base;
pub mod form;
pub mod model;
pub mod style;
pub mod validation;

mod form_value;
mod navigation;

pub use field_base::{BasicField, Checkbox, Select, TextInput};
pub use form::FormResult;
pub use form_value::{FieldSpec, FormValue};
pub use model::{Form, FormExtractError, FormFields, FormFor, FormModel};
pub use ratatui_form_derive::FormModel;
pub use style::FormStyle;
pub use validation::rules::{Email, Ipv4, Ipv6, MaxLength, MinLength, Numeric, Pattern, Required};
pub use validation::{ValidationError, Validator};
