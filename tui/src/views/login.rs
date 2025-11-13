use std::{collections::HashMap, path::PathBuf};

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    prelude::{Constraint, Direction, Layout},
    Frame,
};

use krab_backend::{
    check_user,
    user::{ReadOnlyRecords, User},
};

use crate::{
    centered_absolute_rect,
    components::{
        button::{Button, ButtonConfig},
        input::{Input, InputConfig},
    },
    popups::message::MessagePopup,
    views::{
        home::{Home, Position},
        startup::StartUp,
        View, ViewState,
    },
    Application,
};

/// Represents the login inputs
///
/// # Variants
/// * `Username` - The username field
/// * `MasterPassword` - The master password field
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum LoginInput {
    Username,
    MasterPassword,
}

/// Represents the login buttons
///
/// # Variants
/// * `Confirm` - The confirm button
/// * `Quit` - The quit button
#[derive(Debug, Clone, PartialEq)]
enum LoginButton {
    Confirm,
    Quit,
}

/// Represents the login state
///
/// # Variants
/// * `Username` - The username state
/// * `MasterPassword` - The master password state
/// * `Confirm` - The confirm state
/// * `Quit` - The quit state
#[derive(Debug, Clone, Copy, PartialEq)]
enum LoginState {
    Username,
    MasterPassword,
    Confirm,
    Quit,
}

/// Represents the login view
///
/// # Fields
/// * `username` - The username
/// * `master_password` - The master password
/// * `state` - The state
/// * `path` - The path
/// * `cursors` - The cursors
/// * `input_offsets` - The input offsets
/// * `hidden_password` - Decides if the password is hidden
///
/// # Methods
/// * `new` - Creates a new `Login`
/// * `login` - Logs in the user
/// * `generate_input_config` - Generates the input configuration
/// * `generate_button_config` - Generates the button configuration
///
/// # Implements
/// * `View` - The view trait
#[derive(Debug, Clone)]
pub struct Login {
    username: String,
    master_password: String,
    state: LoginState,
    path: PathBuf,
    cursors: HashMap<LoginInput, u16>,
    input_offsets: HashMap<LoginInput, u16>,
    hidden_password: bool,
}

impl Login {
    /// Creates a new login view
    ///
    /// # Arguments
    /// * `path` - The path
    ///
    /// # Returns
    /// A new `Login` view
    pub fn new(path: &PathBuf) -> Self {
        let mut cursors = HashMap::new();
        let mut input_offsets = HashMap::new();
        cursors.insert(LoginInput::Username, 0);
        cursors.insert(LoginInput::MasterPassword, 0);
        input_offsets.insert(LoginInput::Username, 0);
        input_offsets.insert(LoginInput::MasterPassword, 0);
        Login {
            username: String::new(),
            master_password: String::new(),
            state: LoginState::Username,
            path: path.clone(),
            cursors,
            input_offsets,
            hidden_password: true,
        }
    }

    /// Logs in the user
    ///
    /// # Returns
    /// The user and the read only records if the login is successful
    /// An error message if the login is unsuccessful
    fn login(&self) -> Result<(User, ReadOnlyRecords), String> {
        let user_exists = check_user(&self.username, self.path.clone());
        if !user_exists {
            return Err("Cannot login".to_string());
        }

        let user_creation_result = User::from(&self.path, &self.username, &self.master_password);

        match user_creation_result {
            Ok(u) => Ok(u),
            Err(_) => Err("Cannot login".to_string()),
        }
    }

    /// Generates the input configuration
    ///
    /// # Arguments
    /// * `input` - The input
    ///
    /// # Returns
    /// The input configuration
    fn generate_input_config(&self, input: LoginInput) -> InputConfig {
        match input {
            LoginInput::Username => InputConfig::new(
                self.state == LoginState::Username,
                self.username.clone(),
                false,
                "Username".to_string(),
                if self.state == LoginState::Username {
                    Some(self.cursors.get(&LoginInput::Username).unwrap().clone())
                } else {
                    None
                },
                self.input_offsets
                    .get(&LoginInput::Username)
                    .unwrap()
                    .clone(),
                None,
            ),
            LoginInput::MasterPassword => InputConfig::new(
                self.state == LoginState::MasterPassword,
                self.master_password.clone(),
                self.hidden_password,
                "Master Password | CTRL + s - show/hide".to_string(),
                if self.state == LoginState::MasterPassword {
                    Some(
                        self.cursors
                            .get(&LoginInput::MasterPassword)
                            .unwrap()
                            .clone(),
                    )
                } else {
                    None
                },
                self.input_offsets
                    .get(&LoginInput::MasterPassword)
                    .unwrap()
                    .clone(),
                None,
            ),
        }
    }

    /// Generates the button configuration
    ///
    /// # Arguments
    /// * `button` - The button
    ///
    /// # Returns
    /// The button configuration
    fn generate_button_config(&self, button: LoginButton) -> ButtonConfig {
        match button {
            LoginButton::Confirm => {
                ButtonConfig::new(self.state == LoginState::Confirm, "Confirm".to_string())
            }
            LoginButton::Quit => {
                ButtonConfig::new(self.state == LoginState::Quit, "Quit".to_string())
            }
        }
    }
}

impl View for Login {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        let height = 2 * InputConfig::height() + ButtonConfig::height();
        let width = InputConfig::default_width();
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

        let username_config = self.generate_input_config(LoginInput::Username);
        let master_password_config = self.generate_input_config(LoginInput::MasterPassword);
        let confirm_config = self.generate_button_config(LoginButton::Confirm);
        let quit_config = self.generate_button_config(LoginButton::Quit);
        let mut buffer = f.buffer_mut();

        Input::render(&mut buffer, layout[0], &username_config);
        Input::render(&mut buffer, layout[1], &master_password_config);
        Button::render(&mut buffer, inner_layout[0], &quit_config);
        Button::render(&mut buffer, inner_layout[1], &confirm_config);
    }

    fn handle_key(&mut self, key: &KeyEvent, app: &Application) -> Application {
        let mut app = app.clone();
        let mut change_state = false;

        match self.state {
            LoginState::Username => match key.code {
                KeyCode::Enter | KeyCode::Tab | KeyCode::Down => {
                    self.state = LoginState::MasterPassword;
                }
                KeyCode::Up => {
                    self.state = LoginState::Confirm;
                }
                KeyCode::Esc => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                _ => {
                    let config = self.generate_input_config(LoginInput::Username);
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.username.as_str());
                    self.username = value;
                    self.cursors.insert(LoginInput::Username, cursor_position);
                    self.input_offsets
                        .insert(LoginInput::Username, input_offset);
                }
            },
            LoginState::MasterPassword => match key.code {
                KeyCode::Enter | KeyCode::Tab | KeyCode::Down => {
                    self.state = LoginState::Quit;
                }
                KeyCode::Up => {
                    self.state = LoginState::Username;
                }
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.hidden_password = !self.hidden_password;
                    } else {
                        let config = self.generate_input_config(LoginInput::MasterPassword);
                        let (value, cursor_position, input_offset) =
                            Input::handle_key(key, &config, self.master_password.as_str());
                        self.master_password = value;
                        self.cursors
                            .insert(LoginInput::MasterPassword, cursor_position);
                        self.input_offsets
                            .insert(LoginInput::MasterPassword, input_offset);
                    }
                }
                KeyCode::Esc => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                _ => {
                    let config = self.generate_input_config(LoginInput::MasterPassword);
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.master_password.as_str());
                    self.master_password = value;
                    self.cursors
                        .insert(LoginInput::MasterPassword, cursor_position);
                    self.input_offsets
                        .insert(LoginInput::MasterPassword, input_offset);
                }
            },
            LoginState::Quit => match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                KeyCode::Right
                | KeyCode::Left
                | KeyCode::Tab
                | KeyCode::Char('l')
                | KeyCode::Char('h') => {
                    self.state = LoginState::Confirm;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state = LoginState::MasterPassword;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.state = LoginState::Username;
                }
                _ => {}
            },
            LoginState::Confirm => match key.code {
                KeyCode::Enter => {
                    let res = self.login();
                    match res {
                        Ok((user, ro_records)) => {
                            app.state = ViewState::Home(Home::new(
                                user,
                                ro_records,
                                Position::default(),
                                app.immutable_app_state.rect.unwrap(),
                            ));
                            change_state = true;
                        }
                        Err(_) => {
                            app.mutable_app_state
                                .popups
                                .push(Box::new(MessagePopup::new("Cannot login".to_string())));
                        }
                    }
                }
                KeyCode::Right | KeyCode::Left | KeyCode::Char('l') | KeyCode::Char('h') => {
                    self.state = LoginState::Quit;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state = LoginState::MasterPassword;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => {
                    self.state = LoginState::Username;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
                _ => {}
            },
        }

        if !change_state {
            app.state = ViewState::Login(self.clone());
        }

        app
    }

    fn min_area(&self) -> (u16, u16) {
        let height = 2 * InputConfig::height() + ButtonConfig::height();
        let width = InputConfig::default_width();
        (width * 3, height * 3)
    }
}
