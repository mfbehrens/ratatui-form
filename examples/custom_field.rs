//! Example: a typed form using completely custom field types.
//!
//! The `FormModel` derive knows nothing about `Port` or `Region`; both are
//! wired into the form by implementing the `FormValue` trait. `Port` renders
//! as a [`TextInput`], while `Region` renders as a [`Select`] dropdown.
//!
//! Text fields select their value when focused, so the first keystroke
//! replaces it; press `End` (or an arrow key) first to append instead.
//!
//! Run with: `cargo run --example custom_field`

use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui_form::{FieldSpec, FormFor, FormModel, FormResult, FormValue, Select, TextInput};

/// A custom numeric type backed by a text input.
#[derive(Clone, Debug, PartialEq)]
struct Port(u16);

impl FormValue for Port {
    type FieldType = TextInput;
    fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType {
        let mut input = TextInput::new(spec.label);
        if spec.required {
            input = input.required();
        }
        input.initial_value(value.0.to_string())
    }

    fn form_extract(field: &Self::FieldType) -> Result<Self, String> {
        let raw = field.value();
        let port = raw
            .parse::<u16>()
            .map_err(|_| "expected a port number (0-65535)".to_string())?;
        Ok(Port(port))
    }
}

/// A custom enum type backed by a dropdown.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Region {
    UsEast,
    UsWest,
    EuCentral,
}

impl Region {
    fn as_str(self) -> &'static str {
        match self {
            Region::UsEast => "us-east",
            Region::UsWest => "us-west",
            Region::EuCentral => "eu-central",
        }
    }
}

impl FormValue for Region {
    type FieldType = Select;
    fn form_field(spec: FieldSpec, value: &Self) -> Self::FieldType {
        let mut select = Select::new(spec.label)
            .option("us-east", "US East (Virginia)")
            .option("us-west", "US West (Oregon)")
            .option("eu-central", "EU Central (Frankfurt)");
        if spec.required {
            select = select.required();
        }
        select.initial_value(value.as_str())
    }

    fn form_extract(field: &Self::FieldType) -> Result<Self, String> {
        let val = field.value();
        match val {
            "us-east" => Ok(Region::UsEast),
            "us-west" => Ok(Region::UsWest),
            "eu-central" => Ok(Region::EuCentral),
            other => Err(format!("unknown region: {other}")),
        }
    }
}

/// The model. `Port` and `Region` are used exactly like the built-in types.
#[derive(Debug, FormModel)]
#[form(title = "Server Configuration")]
struct ServerConfig {
    #[form(label = "Server name", required)]
    name: String,

    #[form(label = "Port", required)]
    port: Port,

    #[form(label = "Region", required)]
    region: Region,
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Build the form, seeded with the model's current values.
    let prefill = ServerConfig {
        name: "api-1".into(),
        port: Port(8080),
        region: Region::UsEast,
    };
    let mut form: FormFor<ServerConfig> = prefill.into();

    // Direct typed field access:
    // e.g. form.fields.name, form.fields.port, form.fields.region

    // Main loop. Output is captured and only printed once raw mode is off and
    // the alternate screen is left, otherwise it would be swallowed by the TUI.
    let mut output = String::new();
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            form.render(area, frame.buffer_mut());
        })?;

        if let Event::Key(key_event) = event::read()? {
            // Quick exit with Ctrl+C
            if key_event.code == KeyCode::Char('c')
                && key_event.modifiers.contains(KeyModifiers::CONTROL)
            {
                break;
            }

            form.handle_input(key_event);

            match form.result() {
                FormResult::Submitted => {
                    // Typed extraction: the values are read back through the
                    // `FormValue` impls, so they are fallible.
                    match ServerConfig::try_from(form) {
                        Ok(config) => output.push_str(&format!("Configured:\n{config:?}\n")),
                        Err(errors) => {
                            output.push_str(&format!("Invalid configuration: {errors:?}\n"))
                        }
                    }
                    break;
                }
                FormResult::Cancelled => break,
                FormResult::Active => {}
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    print!("{output}");
    println!("Form exited.");
    Ok(())
}
