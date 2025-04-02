use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{
    centered_absolute_rect,
    components::{
        button::{Button, ButtonConfig},
        input::InputConfig,
    },
    views::{login::Login, register::Register, View, ViewState},
    Application,
};

/// Represents the startup buttons
///
/// # Variants
/// * `Login` - The login button
/// * `Register` - The register button
/// * `Quit` - The quit button
#[derive(Debug, Clone)]
enum StartUpButton {
    Login,
    Register,
    Quit,
}

/// Represents the startup state
///
/// # Variants
/// * `Login` - The login state
/// * `Register` - The register state
/// * `Quit` - The quit state
#[derive(Clone, PartialEq)]
enum StartUpState {
    Login,
    Register,
    Quit,
}

/// Represents the startup view
///
/// # Fields
/// * `state` - The state
///
/// # Methods
/// * `new` - Creates a new `StartUp` view
/// * `generate_button_config` - Generates a button config
///
/// # Implements
/// * `View` - The view trait
#[derive(Clone)]
pub struct StartUp {
    state: StartUpState,
}

impl StartUp {
    /// Creates a new `StartUp`
    ///
    /// # Returns
    /// A new `StartUp`
    pub fn new() -> Self {
        StartUp {
            state: StartUpState::Login,
        }
    }

    /// Generates a button config
    ///
    /// # Arguments
    /// * `button` - The button
    ///
    /// # Returns
    /// A button config
    fn generate_button_config(&self, button: StartUpButton) -> ButtonConfig {
        match button {
            StartUpButton::Login => {
                ButtonConfig::new(self.state == StartUpState::Login, "Login".to_string())
            }
            StartUpButton::Register => {
                ButtonConfig::new(self.state == StartUpState::Register, "Register".to_string())
            }
            StartUpButton::Quit => {
                ButtonConfig::new(self.state == StartUpState::Quit, "Quit".to_string())
            }
        }
    }
}

impl View for StartUp {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        let height = 3 * ButtonConfig::height();
        let width = InputConfig::width();
        let rect = centered_absolute_rect(rect, width, height);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(ButtonConfig::height()),
                Constraint::Length(ButtonConfig::height()),
                Constraint::Length(ButtonConfig::height()),
            ])
            .split(rect);

        let login_config = self.generate_button_config(StartUpButton::Login);
        let register_config = self.generate_button_config(StartUpButton::Register);
        let quit_config = self.generate_button_config(StartUpButton::Quit);
        let mut buffer = f.buffer_mut();

        Button::render(&mut buffer, layout[0], &login_config);
        Button::render(&mut buffer, layout[1], &register_config);
        Button::render(&mut buffer, layout[2], &quit_config);
    }

    fn handle_key(&mut self, key: &KeyEvent, app: &Application) -> Application {
        let mut app = app.clone();
        let mut change_state = false;

        if key.code == KeyCode::Char('q') {
            app.mutable_app_state.running = false;
            return app;
        }

        match self.state {
            StartUpState::Login => match key.code {
                KeyCode::Enter => {
                    app.state = ViewState::Login(Login::new(&app.immutable_app_state.db_path));
                    change_state = true;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => {
                    self.state = StartUpState::Register;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state = StartUpState::Quit;
                }
                _ => {}
            },
            StartUpState::Register => match key.code {
                KeyCode::Enter => {
                    app.state =
                        ViewState::Register(Register::new(&app.immutable_app_state.db_path));
                    change_state = true;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => {
                    self.state = StartUpState::Quit;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state = StartUpState::Login;
                }
                _ => {}
            },
            StartUpState::Quit => match key.code {
                KeyCode::Enter => {
                    app.mutable_app_state.running = false;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => {
                    self.state = StartUpState::Login;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.state = StartUpState::Register;
                }
                _ => {}
            },
        }

        if !change_state {
            app.state = ViewState::StartUp(self.clone());
        }

        app
    }

    fn min_area(&self) -> (u16, u16) {
        (3 * ButtonConfig::height(), InputConfig::width())
    }
}
