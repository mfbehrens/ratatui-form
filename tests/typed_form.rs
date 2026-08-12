//! Integration tests for the `FormModel` derive and `Form`.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_form::{
    BasicField, FieldSpec, FormExtractError, FormModel, FormResult, FormValue, TextInput, TypedForm,
};

#[derive(FormModel, Debug, PartialEq)]
struct Signup {
    #[form(label = "Full Name", required, placeholder = "Ada Lovelace")]
    name: String,

    #[form(label = "Email", required, validate = is_valid_email)]
    email: String,

    #[form(label = "Age")]
    age: u8,

    #[form(label = "Company")]
    company: Option<String>,

    #[form(label = "Subscribe")]
    newsletter: bool,

    #[form(skip)]
    id: u64,
}

fn is_valid_email(value: &str) -> bool {
    value.contains('@')
}

fn sample() -> Signup {
    Signup {
        name: "Alice".into(),
        email: "alice@example.com".into(),
        age: 30,
        company: Some("Acme".into()),
        newsletter: true,
        id: 7,
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_text<T: FormModel>(form: &mut TypedForm<T>, text: &str) {
    for c in text.chars() {
        form.handle_input(key(KeyCode::Char(c)));
    }
}

fn tab<T: FormModel>(form: &mut TypedForm<T>) {
    form.handle_input(key(KeyCode::Tab));
}

#[test]
fn round_trip_preserves_seeded_values() {
    let form = sample().get_form();
    let out = Signup::try_from(form).unwrap();
    assert_eq!(out.name, "Alice");
    assert_eq!(out.email, "alice@example.com");
    assert_eq!(out.age, 30);
    assert_eq!(out.company, Some("Acme".into()));
    assert!(out.newsletter);
    assert_eq!(out.id, 0); // skipped field is Default::default()
}

#[test]
fn into_conversion_from_model() {
    let form: TypedForm<Signup> = sample().into();
    let out = Signup::try_from(form).unwrap();
    assert_eq!(out.name, "Alice");
}

#[test]
fn fields_are_addressed_by_index() {
    let form = sample().get_form();
    assert_eq!(form.value_str(0).as_deref(), Some("Alice"));
    assert_eq!(form.value_str(1).as_deref(), Some("alice@example.com"));
    assert_eq!(form.value_bool(4), Some(true));
    assert_eq!(form.value_str(5), None); // only 5 fields (id is skipped)
}

#[test]
fn editing_fields_changes_extraction() {
    let mut form = sample().get_form();

    // Focus starts on the first field (index 0). The seeded value is selected
    // on focus; pressing End clears the selection so typing appends.
    form.handle_input(key(KeyCode::End));
    type_text(&mut form, " B.");

    // Move to the optional company field (index 3) and clear it.
    tab(&mut form); // email
    tab(&mut form); // age
    tab(&mut form); // company
    for _ in 0.."Acme".len() {
        form.handle_input(key(KeyCode::Backspace));
    }

    let out = Signup::try_from(form).unwrap();
    assert_eq!(out.name, "Alice B.");
    assert_eq!(out.company, None);
}

#[test]
fn empty_optional_extracts_to_none() {
    let model = Signup {
        company: Some(String::new()),
        ..sample()
    };
    let out = Signup::try_from(model.get_form()).unwrap();
    assert_eq!(out.company, None);
}

#[test]
fn required_field_blocks_submit() {
    let model = Signup {
        name: String::new(),
        ..sample()
    };
    let mut form = model.get_form();

    // Tab past all 5 fields to the submit button.
    for _ in 0..5 {
        tab(&mut form);
    }
    form.handle_input(key(KeyCode::Enter));

    assert_eq!(form.result(), &FormResult::Active);
    assert!(!form.validation_errors().is_empty());
}

#[test]
fn validator_function_blocks_submit() {
    let model = Signup {
        email: "not-an-email".into(),
        ..sample()
    };
    let mut form = model.get_form();

    for _ in 0..5 {
        tab(&mut form);
    }
    form.handle_input(key(KeyCode::Enter));

    assert_eq!(form.result(), &FormResult::Active);
    assert!(!form.validation_errors().is_empty());
}

#[test]
fn valid_submit_succeeds() {
    let mut form = sample().get_form();
    for _ in 0..5 {
        tab(&mut form);
    }
    form.handle_input(key(KeyCode::Enter));
    assert_eq!(form.result(), &FormResult::Submitted);
    assert!(form.validation_errors().is_empty());
}

#[test]
fn unparsable_number_fails_extraction() {
    let mut form = sample().get_form();
    tab(&mut form); // email
    tab(&mut form); // age (index 2)
    type_text(&mut form, "abc");

    let err = Signup::try_from(form).unwrap_err();
    assert!(err.iter().any(|e| e.field_index == 2));
}

#[test]
fn form_result_cancelled_on_escape() {
    let mut form = sample().get_form();
    form.handle_input(key(KeyCode::Esc));
    assert_eq!(form.result(), &FormResult::Cancelled);
    assert!(!form.is_active());
}

#[derive(FormModel, Debug, PartialEq)]
struct Network {
    host: String,

    #[form(label = "IP Address")]
    ip: std::net::Ipv4Addr,
}

#[test]
fn ipv4_field_round_trip() {
    let model = Network {
        host: "server-1".into(),
        ip: "10.0.0.5".parse().unwrap(),
    };

    let form = model.get_form();
    assert_eq!(form.value_str(0).as_deref(), Some("server-1"));
    assert_eq!(form.value_str(1).as_deref(), Some("10.0.0.5"));

    let out = Network::try_from(model.get_form()).unwrap();
    assert_eq!(out, model);
}

#[test]
fn ipv4_invalid_blocks_submit() {
    let model = Network {
        host: "server-1".into(),
        ip: "127.0.0.1".parse().unwrap(),
    };
    let mut form = model.get_form();

    tab(&mut form); // ip field (index 1)
    for _ in 0.."127.0.0.1".len() {
        form.handle_input(key(KeyCode::Backspace));
    }
    type_text(&mut form, "999.1.2.3");

    tab(&mut form); // submit
    form.handle_input(key(KeyCode::Enter));

    assert_eq!(form.result(), &FormResult::Active);
    assert!(!form.validation_errors().is_empty());
}

#[test]
fn ipv4_unparsable_fails_extraction() {
    let model = Network {
        host: "server-1".into(),
        ip: "127.0.0.1".parse().unwrap(),
    };
    let mut form = model.get_form();
    tab(&mut form); // ip field (index 1)
    for _ in 0.."127.0.0.1".len() {
        form.handle_input(key(KeyCode::Backspace));
    }
    type_text(&mut form, "nope");

    let err = Network::try_from(form).unwrap_err();
    assert!(err.iter().any(|e| e.field_index == 1));
}

/// A completely custom field type: the derive knows nothing about it and
/// relies entirely on the `FormValue` implementation.
#[derive(Clone, Debug, PartialEq)]
struct Department(String);

impl FormValue for Department {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn BasicField> {
        Box::new(TextInput::new(spec.label).initial_value(value.0.clone()))
    }

    fn form_extract<M: FormModel>(
        form: &TypedForm<M>,
        index: usize,
    ) -> Result<Self, FormExtractError> {
        let value = form.value_str(index).ok_or_else(|| FormExtractError {
            field_index: index,
            message: "field not found in form".to_string(),
        })?;
        Ok(Department(value))
    }
}

#[derive(FormModel, Debug, PartialEq)]
struct Employee {
    name: String,
    department: Department,
}

#[test]
fn custom_type_round_trip() {
    let model = Employee {
        name: "Grace".into(),
        department: Department("Engineering".into()),
    };

    let form = model.get_form();
    assert_eq!(form.value_str(1).as_deref(), Some("Engineering"));

    let out = Employee::try_from(form).unwrap();
    assert_eq!(out, model);
}

#[test]
fn custom_type_editing() {
    let mut form = Employee {
        name: "Grace".into(),
        department: Department("Engineering".into()),
    }
    .get_form();

    tab(&mut form); // department (index 1)
                    // The seeded value is selected on focus; press End to append.
    form.handle_input(key(KeyCode::End));
    type_text(&mut form, " Ops");

    let out = Employee::try_from(form).unwrap();
    assert_eq!(out.department.0, "Engineering Ops");
}

#[derive(Clone, Debug, PartialEq)]
struct Port(u16);

impl FormValue for Port {
    fn form_field(spec: FieldSpec, value: &Self) -> Box<dyn BasicField> {
        Box::new(TextInput::new(spec.label).initial_value(value.0.to_string()))
    }

    fn form_extract<M: FormModel>(
        form: &TypedForm<M>,
        index: usize,
    ) -> Result<Self, FormExtractError> {
        let value = form.value_str(index).ok_or_else(|| FormExtractError {
            field_index: index,
            message: "field not found in form".to_string(),
        })?;
        let port = value.parse().map_err(|_| FormExtractError {
            field_index: index,
            message: "expected a port".to_string(),
        })?;
        Ok(Port(port))
    }
}

#[derive(FormModel, Debug, PartialEq)]
struct Server {
    host: String,
    port: Port,
}

#[test]
fn custom_type_extraction_failure() {
    let mut form = Server {
        host: "localhost".into(),
        port: Port(8080),
    }
    .get_form();

    tab(&mut form); // port (index 1)
    for _ in 0.."8080".len() {
        form.handle_input(key(KeyCode::Backspace));
    }
    type_text(&mut form, "abc");

    let err = Server::try_from(form).unwrap_err();
    assert!(err.iter().any(|e| e.field_index == 1));
}

#[test]
fn custom_type_replaces_seeded_value_on_focus() {
    let mut form = Server {
        host: "localhost".into(),
        port: Port(8080),
    }
    .get_form();

    tab(&mut form); // port (index 1)
                    // The seeded value is selected on focus, so the first keystroke replaces
                    // it instead of appending (previously "8080" + typed digits overflowed u16).
    type_text(&mut form, "9000");

    let out = Server::try_from(form).unwrap();
    assert_eq!(out.port, Port(9000));
}

#[derive(FormModel, Debug, PartialEq)]
struct Profile {
    username: String,

    #[form(label = "Company", required)]
    company: Option<String>,
}

#[test]
fn required_optional_field_blocks_submit() {
    let model = Profile {
        username: "ada".into(),
        company: Some(String::new()),
    };
    let mut form = model.get_form();

    // Tab past both fields to the submit button.
    tab(&mut form); // company
    tab(&mut form); // submit button
    form.handle_input(key(KeyCode::Enter));

    assert_eq!(form.result(), &FormResult::Active);
    assert!(!form.validation_errors().is_empty());
}
