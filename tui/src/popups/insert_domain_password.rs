use std::collections::HashMap;

use krab_backend::generate_password;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::{Constraint, Direction, Layout, Rect},
    widgets::Clear,
    Frame,
};

use crate::{
    centered_absolute_rect,
    components::{
        button::{Button, ButtonConfig},
        input::{Input, InputConfig},
    },
    popups::{Popup, PopupType},
    Application,
};

/// Represents the domain password input fields
///
/// # Variants
/// * `Username` - The username field
/// * `MasterPassword` - The master password field
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum DomainPasswordInput {
    Domain,
    Password,
}

/// Represents the buttons in the insert domain password popup
///
/// # Variants
/// * `Confirm` - The confirm button
/// * `Quit` - The quit button
#[derive(Debug, Clone, PartialEq)]
enum DomainPasswordButton {
    Confirm,
    Quit,
}

/// Represents the state of the insert domain password popup
///
/// # Variants
/// * `Domain` - The domain state
/// * `Password` - The password state
/// * `Confirm` - The confirm state
/// * `Quit` - The quit state
#[derive(Clone, PartialEq)]
pub enum InsertDomainPasswordState {
    Domain,
    Password,
    Confirm,
    Quit,
}

/// Represents the exit state of the insert domain password popup
///
/// # Variants
/// * `Confirm` - The confirm state
/// * `Quit` - The quit state
#[derive(Clone, PartialEq)]
pub enum InsertDomainPasswordExitState {
    Confirm,
    Quit,
}

/// Represents the insert domain password popup
///
/// # Fields
/// * `domain` - The domain
/// * `password` - The password
/// * `state` - The state
/// * `exit_state` - The exit state
/// * `cursors` - The cursors
/// * `input_offsets` - The input offsets
///
/// # Methods
/// * `new` - Creates a new `InsertDomainPassword`
/// * `exit_state` - Returns the exit state of the popup
/// * `domain` - Returns the domain of the popup
/// * `password` - Returns the password of the popup
/// * `min_area` - Returns the minimum area of the popup
/// * `generate_input_config` - Generates the input config for the popup
/// * `generate_button_config` - Generates the button config for the popup
///
/// # Implements
/// * `Popup` - The popup trait
#[derive(Clone)]
pub struct InsertDomainPassword {
    domain: String,
    password: String,
    state: InsertDomainPasswordState,
    exit_state: Option<InsertDomainPasswordExitState>,
    cursors: HashMap<DomainPasswordInput, u16>,
    input_offsets: HashMap<DomainPasswordInput, u16>,
}

impl InsertDomainPassword {
    /// Creates a new insert domain password popup
    ///
    /// # Returns
    /// A new `InsertDomainPassword`
    pub fn new() -> Self {
        let mut cursors = HashMap::new();
        let mut input_offsets = HashMap::new();
        cursors.insert(DomainPasswordInput::Domain, 0);
        cursors.insert(DomainPasswordInput::Password, 0);
        input_offsets.insert(DomainPasswordInput::Domain, 0);
        input_offsets.insert(DomainPasswordInput::Password, 0);
        InsertDomainPassword {
            domain: String::new(),
            password: String::new(),
            state: InsertDomainPasswordState::Domain,
            exit_state: None,
            cursors,
            input_offsets,
        }
    }

    /// Returns the exit state of the popup
    ///
    /// # Returns
    /// An `Option<InsertDomainPasswordExitState>` representing the exit state of the popup
    pub fn exit_state(&self) -> Option<InsertDomainPasswordExitState> {
        self.exit_state.clone()
    }

    /// Returns the domain of the popup
    ///
    /// # Returns
    /// A `String` representing the domain of the popup
    pub fn domain(&self) -> String {
        self.domain.clone()
    }

    /// Returns the password of the popup
    ///
    /// # Returns
    /// A `String` representing the password of the popup
    pub fn password(&self) -> String {
        self.password.clone()
    }
    
    /// Returns the minimum area of the popup
    /// 
    /// # Returns
    /// A tuple representing the minimum area of the popup
    pub fn min_area() -> (u16, u16) {
        let height = 2 * InputConfig::height() + ButtonConfig::height();
        let width = InputConfig::width();
        (width, height)
    }

    /// Generates the input config for the popup
    /// 
    /// # Arguments
    /// * `input` - The input to generate the config for
    /// 
    /// # Returns
    /// An `InputConfig` representing the input config for the popup
    fn generate_input_config(&self, input: DomainPasswordInput) -> InputConfig {
        match input {
            DomainPasswordInput::Domain => InputConfig::new(
                self.state == InsertDomainPasswordState::Domain,
                self.domain.clone(),
                false,
                "Domain".to_string(),
                if self.state == InsertDomainPasswordState::Domain {
                    Some(
                        self.cursors
                            .get(&DomainPasswordInput::Domain)
                            .unwrap()
                            .clone(),
                    )
                } else {
                    None
                },
                self.input_offsets
                    .get(&DomainPasswordInput::Domain)
                    .unwrap()
                    .clone(),
            ),
            DomainPasswordInput::Password => InputConfig::new(
                self.state == InsertDomainPasswordState::Password,
                self.password.clone(),
                true,
                "Password".to_string(),
                if self.state == InsertDomainPasswordState::Password {
                    Some(
                        self.cursors
                            .get(&DomainPasswordInput::Password)
                            .unwrap()
                            .clone(),
                    )
                } else {
                    None
                },
                self.input_offsets
                    .get(&DomainPasswordInput::Password)
                    .unwrap()
                    .clone(),
            ),
        }
    }

    /// Generates the button config for the popup
    /// 
    /// # Arguments
    /// * `input` - The input to generate the config for
    /// 
    /// # Returns
    /// A `ButtonConfig` representing the button config for the popup
    fn generate_button_config(&self, input: DomainPasswordButton) -> ButtonConfig {
        match input {
            DomainPasswordButton::Confirm => ButtonConfig::new(
                self.state == InsertDomainPasswordState::Confirm,
                "Confirm".to_string(),
            ),
            DomainPasswordButton::Quit => ButtonConfig::new(
                self.state == InsertDomainPasswordState::Quit,
                "Quit".to_string(),
            ),
        }
    }
}

impl Popup for InsertDomainPassword {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        let height = 2 * InputConfig::height() + ButtonConfig::height();
        let width = InputConfig::width();
        let rect = centered_absolute_rect(rect, width, height);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(InputConfig::height()),
                Constraint::Length(InputConfig::height()),
                Constraint::Length(ButtonConfig::height()),
            ])
            .split(rect);

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(layout[2]);

        let domain_config = self.generate_input_config(DomainPasswordInput::Domain);
        let password_config = self.generate_input_config(DomainPasswordInput::Password);

        let confirm_config = self.generate_button_config(DomainPasswordButton::Confirm);
        let quit_config = self.generate_button_config(DomainPasswordButton::Quit);
        f.render_widget(Clear, rect);
        let mut buffer = f.buffer_mut();

        Input::render(&mut buffer, layout[0], &domain_config);
        Input::render(&mut buffer, layout[1], &password_config);
        Button::render(&mut buffer, inner_layout[0], &quit_config);
        Button::render(&mut buffer, inner_layout[1], &confirm_config);
    }

    fn handle_key(
        &mut self,
        key: &KeyEvent,
        app: &Application,
    ) -> (Application, Option<Box<dyn Popup>>) {
        let mut app = app.clone();
        let mut poped = false;

        match self.state {
            InsertDomainPasswordState::Domain => match key.code {
                KeyCode::Up => {
                    self.state = InsertDomainPasswordState::Quit;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Enter => {
                    self.state = InsertDomainPasswordState::Password;
                }
                _ => {
                    let config = self.generate_input_config(DomainPasswordInput::Domain);
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.domain());
                    self.domain = value;
                    self.cursors
                        .insert(DomainPasswordInput::Domain, cursor_position);
                    self.input_offsets
                        .insert(DomainPasswordInput::Domain, input_offset);
                }
            },
            InsertDomainPasswordState::Password => match key.code {
                KeyCode::Char('g') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.password = generate_password();
                    } else {
                        let config = self.generate_input_config(DomainPasswordInput::Password);
                        let (value, cursor_position, input_offset) =
                            Input::handle_key(key, &config, self.password());
                        self.password = value;
                        self.cursors
                            .insert(DomainPasswordInput::Password, cursor_position);
                        self.input_offsets
                            .insert(DomainPasswordInput::Password, input_offset);
                    }
                }
                KeyCode::Up => {
                    self.state = InsertDomainPasswordState::Domain;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Enter => {
                    self.state = InsertDomainPasswordState::Quit;
                }
                _ => {
                    let config = self.generate_input_config(DomainPasswordInput::Password);
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.password());
                    self.password = value;
                    self.cursors
                        .insert(DomainPasswordInput::Password, cursor_position);
                    self.input_offsets
                        .insert(DomainPasswordInput::Password, input_offset);
                }
            },
            InsertDomainPasswordState::Quit => match key.code {
                KeyCode::Enter => {
                    app.mutable_app_state.popups.pop();
                    self.exit_state = Some(InsertDomainPasswordExitState::Quit);
                    poped = true;
                }
                KeyCode::Up => {
                    self.state = InsertDomainPasswordState::Password;
                }
                KeyCode::Right | KeyCode::Tab | KeyCode::Left => {
                    self.state = InsertDomainPasswordState::Confirm;
                }
                KeyCode::Down => {
                    self.state = InsertDomainPasswordState::Domain;
                }
                _ => {}
            },
            InsertDomainPasswordState::Confirm => match key.code {
                KeyCode::Enter => {
                    app.mutable_app_state.popups.pop();
                    self.exit_state = Some(InsertDomainPasswordExitState::Confirm);
                    poped = true;
                }
                KeyCode::Left | KeyCode::Right => {
                    self.state = InsertDomainPasswordState::Quit;
                }
                KeyCode::Down | KeyCode::Tab => {
                    self.state = InsertDomainPasswordState::Domain;
                }
                KeyCode::Up => {
                    self.state = InsertDomainPasswordState::Password;
                }
                _ => {}
            },
        }

        if !poped {
            app.mutable_app_state.popups.pop();
            app.mutable_app_state.popups.push(Box::new(self.clone()));
            return (app, None);
        }

        (app, Some(Box::new(self.clone())))
    }

    fn wrapper(&self, rect: Rect) -> Rect {
        centered_absolute_rect(
            rect,
            InputConfig::width(),
            InputConfig::height() * 2 + ButtonConfig::height(),
        )
    }

    fn popup_type(&self) -> PopupType {
        PopupType::InsertDomainPassword
    }
}
