# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Text fields select their value when they gain focus, so the first keystroke
  replaces it instead of appending. Press `End` or an arrow key to append.
  This fixes editing pre-filled numeric fields, which previously produced
  out-of-range values like `"8080" + "123"`.

### Changed
- The `FormModel` derive no longer hardcodes a type→field mapping. Field
  construction and extraction are now delegated to the new `FormValue` trait,
  so completely custom field types can be added by implementing `FormValue`.
  Built-in implementations cover `String`, `Option<String>`, `bool`,
  `std::net::Ipv4Addr`, `std::net::Ipv6Addr`, and all numeric types.
- Auto-generated labels now uppercase short initialisms in field names
  (`ip` → `IP`, `api_key` → `API KEY`).
- README rewritten to document the derive-based API and custom field types.
- Removed the stale VHS demo tape, demo GIF, and leftover `signup.json`.
- Removed unused public API: `Field::label`, `Field::is_required`,
  `Select::options`, `FocusManager::focus_submit` (and the `navigation`
  module is now private). `FormEngine` is no longer public.
- Fixed `#[form(required)]` being silently ignored on `Option<String>` fields.

## [0.1.1] - 2025-01-31

### Changed
- Renamed crate from `tform` to `ratatui-form` to avoid confusion with the
  unrelated `tform` crate on crates.io. Users of the previous name should
  update their `Cargo.toml` dependency to `ratatui-form`.

## [0.1.0] - 2025-01-31

### Added
- Initial release.
- Fluent `Form::builder()` API with `.text()`, `.select()`, `.checkbox()`,
  `.block()`, `.title()`, and `.style()` methods.
- Field types: `TextInput`, `Select` (dropdown), `Checkbox`.
- Composite blocks: `AddressBlock`, `ContactBlock`, `DateRangeBlock`.
- Built-in validators: `Required`, `Email`, `MinLength`, `MaxLength`,
  `Pattern` (regex), plus pre-built `Pattern::zip_code()`, `Pattern::phone()`,
  `Pattern::date()`.
- Custom validators via the `Validator` trait.
- Keyboard navigation: Tab / Shift+Tab / Arrow keys / Enter / Space / Esc,
  plus Ctrl+A, Ctrl+E, Ctrl+U in text inputs.
- Theming via `FormStyle` with `dark()` and `light()` presets and fluent
  per-component overrides.
- JSON export via `Form::to_json()` and `Form::write_json()`.

[Unreleased]: https://github.com/DavidLiedle/ratatui-form/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/DavidLiedle/ratatui-form/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/DavidLiedle/ratatui-form/releases/tag/v0.1.0
