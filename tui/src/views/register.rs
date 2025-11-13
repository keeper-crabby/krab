use std::{collections::HashMap, path::PathBuf};

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use krab_backend::user::{RecordOperationConfig, User};

use crate::{
    centered_absolute_rect,
    components::{
        button::{Button, ButtonConfig},
        input::{Input, InputConfig},
    },
    popups::{
        insert_domain_password::{InsertDomainPassword, InsertDomainPasswordExitState},
        message::MessagePopup,
        Popup,
    },
    views::{startup::StartUp, ViewState},
    Application, View,
};

/// Represents the register inputs  
///
/// # Variants
///
/// * `Username` - The username state
/// * `MasterPassword` - The master password state
/// * `ConfirmMasterPassword` - The confirm master password state
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum RegisterInput {
    Username,
    MasterPassword,
    ConfirmMasterPassword,
}

/// Represents the register buttons
///
/// # Variants
/// * `Confirm` - The confirm button
/// * `Quit` - The quit button
#[derive(Debug, Clone, PartialEq)]
enum RegisterButton {
    Confirm,
    Quit,
}

/// Represents the register state
///
/// # Fields
/// * `username` - The username
/// * `master_password` - The master password
/// * `confirm_master_password` - The confirm master password
/// * `Confirm` - The confirm state
/// * `Quit` - The quit state
#[derive(Debug, Clone, Copy, PartialEq)]
enum RegisterState {
    Username,
    MasterPassword,
    ConfirmMasterPassword,
    Confirm,
    Quit,
}

/// Represents the register state
///
/// # Fields
/// * `username` - The username
/// * `master_password` - The master password
/// * `confirm_master_password` - The master password confirmation
/// * `state` - The state
/// * `domain` - The domain
/// * `password` - The password
/// * `path` - The path
/// * `cursors` - The cursors
/// * `input_offsets` - The input offsets
/// * `hidden_password` - Decides if the master password is hidden
/// * `hidden_confirm_password` - Decides if the confirm master password is hidden
///
/// # Methods
/// * `new` - Creates a new `Register`
/// * `generate_input_config` - Generates an input configuration
/// * `generate_button_config` - Generates a button configuration
///
/// # Implements
/// * `View` - The view trait
#[derive(Debug, Clone, PartialEq)]
pub struct Register {
    username: String,
    master_password: String,
    confirm_master_password: String,
    state: RegisterState,
    domain: String,
    password: String,
    path: PathBuf,
    cursors: HashMap<RegisterInput, u16>,
    input_offsets: HashMap<RegisterInput, u16>,
    hidden_password: bool,
    hidden_confirm_password: bool,
}

impl Register {
    /// Creates a new register view
    ///
    /// # Arguments
    /// * `path` - The path
    ///
    /// # Returns
    /// A new `Register`
    pub fn new(path: &PathBuf) -> Self {
        let mut cursors = HashMap::new();
        let mut input_offsets = HashMap::new();
        cursors.insert(RegisterInput::Username, 0);
        cursors.insert(RegisterInput::MasterPassword, 0);
        cursors.insert(RegisterInput::ConfirmMasterPassword, 0);
        input_offsets.insert(RegisterInput::Username, 0);
        input_offsets.insert(RegisterInput::MasterPassword, 0);
        input_offsets.insert(RegisterInput::ConfirmMasterPassword, 0);
        Register {
            username: String::new(),
            master_password: String::new(),
            confirm_master_password: String::new(),
            state: RegisterState::Username,
            domain: String::new(),
            password: String::new(),
            path: path.clone(),
            cursors,
            input_offsets,
            hidden_password: true,
            hidden_confirm_password: true,
        }
    }

    /// Generates an input configuration
    ///
    /// # Arguments
    /// * `input` - The input
    ///
    /// # Returns
    /// An input configuration
    fn generate_input_config(&self, input: RegisterInput) -> InputConfig {
        match input {
            RegisterInput::Username => InputConfig::new(
                self.state == RegisterState::Username,
                self.username.clone(),
                false,
                "Username".to_string(),
                if self.state == RegisterState::Username {
                    Some(self.cursors.get(&RegisterInput::Username).unwrap().clone())
                } else {
                    None
                },
                self.input_offsets
                    .get(&RegisterInput::Username)
                    .unwrap()
                    .clone(),
                None,
            ),
            RegisterInput::MasterPassword => InputConfig::new(
                self.state == RegisterState::MasterPassword,
                self.master_password.clone(),
                self.hidden_password,
                "Master Password | CTRL + s - show/hide".to_string(),
                if self.state == RegisterState::MasterPassword {
                    Some(
                        self.cursors
                            .get(&RegisterInput::MasterPassword)
                            .unwrap()
                            .clone(),
                    )
                } else {
                    None
                },
                self.input_offsets
                    .get(&RegisterInput::MasterPassword)
                    .unwrap()
                    .clone(),
                None,
            ),
            RegisterInput::ConfirmMasterPassword => InputConfig::new(
                self.state == RegisterState::ConfirmMasterPassword,
                self.confirm_master_password.clone(),
                self.hidden_confirm_password,
                "Confirm Master Password | CTRL + s - show/hide".to_string(),
                if self.state == RegisterState::ConfirmMasterPassword {
                    Some(
                        self.cursors
                            .get(&RegisterInput::ConfirmMasterPassword)
                            .unwrap()
                            .clone(),
                    )
                } else {
                    None
                },
                self.input_offsets
                    .get(&RegisterInput::ConfirmMasterPassword)
                    .unwrap()
                    .clone(),
                None,
            ),
        }
    }

    /// Generates a button configuration
    ///
    /// # Arguments
    /// * `button` - The button
    ///
    /// # Returns
    /// A button configuration
    fn generate_button_config(&self, button: RegisterButton) -> ButtonConfig {
        match button {
            RegisterButton::Confirm => {
                ButtonConfig::new(self.state == RegisterState::Confirm, "Confirm".to_string())
            }
            RegisterButton::Quit => {
                ButtonConfig::new(self.state == RegisterState::Quit, "Quit".to_string())
            }
        }
    }
}

impl View for Register {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        let height = 3 * InputConfig::height() + ButtonConfig::height();
        let width = InputConfig::default_width();
        let rect = centered_absolute_rect(rect, width, height);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(InputConfig::height()),
                Constraint::Length(InputConfig::height()),
                Constraint::Length(InputConfig::height()),
                Constraint::Length(ButtonConfig::height()),
            ])
            .split(rect);

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(layout[3]);

        let username_config = self.generate_input_config(RegisterInput::Username);
        let master_password_config = self.generate_input_config(RegisterInput::MasterPassword);
        let confirm_master_password_config =
            self.generate_input_config(RegisterInput::ConfirmMasterPassword);
        let confirm_config = self.generate_button_config(RegisterButton::Confirm);
        let quit_config = self.generate_button_config(RegisterButton::Quit);
        let mut buffer = f.buffer_mut();

        Input::render(&mut buffer, layout[0], &username_config);
        Input::render(&mut buffer, layout[1], &master_password_config);
        Input::render(&mut buffer, layout[2], &confirm_master_password_config);
        Button::render(&mut buffer, inner_layout[0], &quit_config);
        Button::render(&mut buffer, inner_layout[1], &confirm_config);
    }

    fn handle_key(&mut self, key: &KeyEvent, app: &Application) -> Application {
        let mut app = app.clone();
        let mut change_state = false;

        match self.state {
            RegisterState::Username => match key.code {
                KeyCode::Enter | KeyCode::Tab | KeyCode::Down => {
                    self.state = RegisterState::MasterPassword;
                }
                KeyCode::Up => {
                    self.state = RegisterState::Confirm;
                }
                KeyCode::Esc => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                _ => {
                    let config = self.generate_input_config(RegisterInput::Username);
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.username.as_str());
                    self.username = value;
                    self.cursors
                        .insert(RegisterInput::Username, cursor_position);
                    self.input_offsets
                        .insert(RegisterInput::Username, input_offset);
                }
            },
            RegisterState::MasterPassword => match key.code {
                KeyCode::Enter | KeyCode::Tab | KeyCode::Down => {
                    self.state = RegisterState::ConfirmMasterPassword;
                }
                KeyCode::Up => {
                    self.state = RegisterState::Username;
                }
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.hidden_password = !self.hidden_password;
                    } else {
                        let config = self.generate_input_config(RegisterInput::MasterPassword);
                        let (value, cursor_position, input_offset) =
                            Input::handle_key(key, &config, self.master_password.as_str());
                        self.master_password = value;
                        self.cursors
                            .insert(RegisterInput::MasterPassword, cursor_position);
                        self.input_offsets
                            .insert(RegisterInput::MasterPassword, input_offset);
                    }
                }
                KeyCode::Esc => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                _ => {
                    let config = self.generate_input_config(RegisterInput::MasterPassword);
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.master_password.as_str());
                    self.master_password = value;
                    self.cursors
                        .insert(RegisterInput::MasterPassword, cursor_position);
                    self.input_offsets
                        .insert(RegisterInput::MasterPassword, input_offset);
                }
            },
            RegisterState::ConfirmMasterPassword => match key.code {
                KeyCode::Enter | KeyCode::Tab | KeyCode::Down => {
                    self.state = RegisterState::Quit;
                }
                KeyCode::Up => {
                    self.state = RegisterState::MasterPassword;
                }
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.hidden_confirm_password = !self.hidden_confirm_password;
                    } else {
                        let config = self.generate_input_config(RegisterInput::ConfirmMasterPassword);
                        let (value, cursor_position, input_offset) =
                            Input::handle_key(key, &config, self.confirm_master_password.as_str());
                        self.confirm_master_password = value;
                        self.cursors
                            .insert(RegisterInput::ConfirmMasterPassword, cursor_position);
                        self.input_offsets
                            .insert(RegisterInput::ConfirmMasterPassword, input_offset);
                    }
                }
                KeyCode::Esc => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                _ => {
                    let config = self.generate_input_config(RegisterInput::ConfirmMasterPassword);
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.confirm_master_password.as_str());
                    self.confirm_master_password = value;
                    self.cursors
                        .insert(RegisterInput::ConfirmMasterPassword, cursor_position);
                    self.input_offsets
                        .insert(RegisterInput::ConfirmMasterPassword, input_offset);
                }
            },
            RegisterState::Quit => match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                KeyCode::Right
                | KeyCode::Left
                | KeyCode::Tab
                | KeyCode::Char('h')
                | KeyCode::Char('l') => {
                    self.state = RegisterState::Confirm;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state = RegisterState::ConfirmMasterPassword;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.state = RegisterState::Username;
                }
                _ => {}
            },
            RegisterState::Confirm => match key.code {
                KeyCode::Enter => {
                    app.mutable_app_state
                        .popups
                        .push(Box::new(InsertDomainPassword::new()));
                    change_state = true;
                }
                KeyCode::Right | KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('l') => {
                    self.state = RegisterState::Quit;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state = RegisterState::ConfirmMasterPassword;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => {
                    self.state = RegisterState::Username;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                _ => {}
            },
        }

        if !change_state {
            app.state = ViewState::Register(self.clone());
        }

        app
    }

    fn handle_insert_record_popup(
        &mut self,
        app: Application,
        _popup: Box<dyn Popup>,
    ) -> Application {
        if self.master_password != self.confirm_master_password {
            let mut app = app.clone();
            app.mutable_app_state
                .popups
                .push(Box::new(MessagePopup::new(
                    "Could not create user.".to_string(),
                )));
            return app;
        }

        let domain: String;
        let password: String;
        let insert_password = _popup.downcast::<InsertDomainPassword>();

        match insert_password {
            Ok(insert_password) => {
                if insert_password.exit_state() == Some(InsertDomainPasswordExitState::Quit) {
                    return app;
                }
                domain = insert_password.domain();
                password = insert_password.password();
            }
            Err(_) => {
                unreachable!();
            }
        }

        let mut app = app.clone();

        let config = RecordOperationConfig::new(
            &self.username,
            &self.master_password,
            &domain,
            &password,
            &self.path,
        );

        let res = User::new(&config);

        match res {
            Ok(_) => {
                app.state = ViewState::StartUp(StartUp::new());
            }
            Err(_) => {
                app.mutable_app_state
                    .popups
                    .push(Box::new(MessagePopup::new(
                        "Could not create user.".to_string(),
                    )));
            }
        }

        app
    }

    fn min_area(&self) -> (u16, u16) {
        let height = 3 * InputConfig::height() + ButtonConfig::height();
        let width = InputConfig::default_width();
        (width * 3, height * 3)
    }
}
