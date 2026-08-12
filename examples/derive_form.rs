//! Example: typed form built from a struct via `#[derive(FormModel)]`.
//!
//! Run with: `cargo run --example derive_form`

use std::{io, net::Ipv6Addr};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui_form::{Form, FormModel, FormResult};

/// A simple signup model. Each field maps to a text input or checkbox;
/// `#[form(skip)]` fields are excluded from the form.
#[derive(Debug, FormModel)]
#[form(title = "Sign Up")]
#[allow(dead_code)] // `id` is skipped from the form on purpose
struct Signup {
    #[form(label = "Full Name", required, placeholder = "Ada Lovelace")]
    name: String,

    #[form(label = "Email", required, validate = is_valid_email)]
    email: String,

    #[form(label = "Age")]
    age: u8,

    #[form(label = "IP address")]
    ip: Ipv6Addr,

    #[form(label = "Company")]
    company: Option<String>,

    #[form(label = "Subscribe to newsletter")]
    newsletter: bool,

    #[form(skip)]
    id: u64,
}

/// Validator functions are plain `fn(&str) -> bool`.
fn is_valid_email(value: &str) -> bool {
    value.contains('@') && value.contains('.')
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Build the form, seeded with the model's current values.
    let prefill = Signup {
        name: "Ada".into(),
        email: "ada@example.com".into(),
        age: 37,
        ip: Ipv6Addr::new(192, 168, 0, 1, 4, 24, 2, 4),
        company: Some("Analytical Engine".into()),
        newsletter: true,
        id: 42,
    };
    let mut form = Form::<Signup>::from(prefill);

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
                    // Typed extraction: the fields are guaranteed to exist, but
                    // values may still fail to parse, so it's fallible.
                    match Signup::try_from(form) {
                        Ok(signup) => output.push_str(&format!("Sign up:\n{:?}\n", signup)),
                        Err(errors) => output.push_str(&format!("Invalid signup: {errors:?}\n")),
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
