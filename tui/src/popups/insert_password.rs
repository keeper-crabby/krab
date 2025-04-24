use krab_backend::generate_password;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Alignment,
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};

use crate::{
    centered_absolute_rect,
    components::{
        button::{Button, ButtonConfig},
        input::{Input, InputConfig},
    },
    from,
    popups::{Popup, PopupType},
    Application, COLOR_RED, COLOR_WHITE,
};

/// Represents the buttons in the insert password popup
///
/// # Variants
/// * `Confirm` - The confirm button
/// * `Quit` - The quit button
#[derive(Debug, Clone, PartialEq)]
enum PasswordButton {
    Confirm,
    Quit,
}

/// Represents the state of the insert password popup
///
/// # Variants
/// * `password` - The password state
/// * `Confirm` - The confirm state
/// * `Quit` - The quit state
#[derive(Clone, PartialEq)]
pub enum InsertPasswordState {
    Password,
    Confirm,
    Quit,
}

/// Represents the exit state of the insert password popup
///
/// # Variants
/// * `Confirm` - The confirm state
/// * `Quit` - The quit state
#[derive(Clone, PartialEq)]
pub enum InsertPasswordExitState {
    Confirm,
    Quit,
}

/// Represents the insert password popup
///
/// # Fields
/// * `password` - The password
/// * `state` - The state
/// * `exit_state` - The exit state
/// * `cursors` - The cursors
/// * `input_offsets` - The input offsets
///
/// # Methods
/// * `new` - Creates a new `InsertPassword`
/// * `password` - Returns the password
/// * `exit_state` - Returns the exit state
/// * `min_area` - Returns the minimum area of the popup
/// * `generate_input_config` - Returns the input config for the popup
/// * `generate_button_config` - Returns the button config for the popup
///
/// # Implements
/// * `Popup` - The popup trait
#[derive(Clone)]
pub struct InsertPassword {
    domain: String,
    password: String,
    state: InsertPasswordState,
    exit_state: Option<InsertPasswordExitState>,
    cursor: u16,
    input_offset: u16,
}

impl InsertPassword {
    /// Creates a new insert password popup
    ///
    /// # Returns
    /// A new `InsertPassword`
    pub fn new(domain: String) -> Self {
        let cursor = 0;
        let input_offset = 0;
        InsertPassword {
            domain,
            password: String::new(),
            state: InsertPasswordState::Password,
            exit_state: None,
            cursor,
            input_offset,
        }
    }

    /// Returns the password password
    ///
    /// # Returns
    /// The password password
    pub fn password(&self) -> String {
        self.password.clone()
    }

    /// Returns the state of the popup
    ///
    /// # Returns
    /// The state of the popup
    pub fn exit_state(&self) -> Option<InsertPasswordExitState> {
        self.exit_state.clone()
    }

    /// Returns the minimum area of the popup
    ///
    /// # Returns
    /// The minimum area of the popup
    pub fn min_area() -> (u16, u16) {
        (
            InputConfig::width(),
            InputConfig::height() * 2 + ButtonConfig::height(),
        )
    }

    /// Returns the maximum area of the popup
    ///
    /// # Returns
    /// The maximum area of the popup
    fn generate_input_config(&self) -> InputConfig {
        InputConfig::new(
            self.state == InsertPasswordState::Password,
            self.password(),
            true,
            "Password | CTRL + g - generate".to_string(),
            if self.state == InsertPasswordState::Password {
                Some(self.cursor)
            } else {
                None
            },
            self.input_offset,
        )
    }

    /// Returns the button config for the given input
    ///
    /// # Arguments
    /// * `input` - The input
    ///
    /// # Returns
    /// The button config for the given input
    fn generate_button_config(&self, input: PasswordButton) -> ButtonConfig {
        match input {
            PasswordButton::Confirm => ButtonConfig::new(
                self.state == InsertPasswordState::Confirm,
                "Confirm".to_string(),
            ),
            PasswordButton::Quit => {
                ButtonConfig::new(self.state == InsertPasswordState::Quit, "Quit".to_string())
            }
        }
    }
}

impl Popup for InsertPassword {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        let height = InputConfig::height() * 2 + ButtonConfig::height();
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

        let password_config = self.generate_input_config();

        let confirm_config = self.generate_button_config(PasswordButton::Confirm);
        let quit_config = self.generate_button_config(PasswordButton::Quit);
        f.render_widget(Clear, rect);
        let mut buffer = f.buffer_mut();

        // - 4 is for the padding which InputConfig::width() adds, not the best approach but works for now
        let domain = if self.domain.len() > InputConfig::width() as usize - 4 {
            self.domain
                .chars()
                .take(InputConfig::width() as usize - 3 - 4)
                .collect::<String>()
                + "..."
        } else {
            self.domain.clone()
        };
        let domain = Text::from(Line::from(format!(" {} ", domain)))
            .style(Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White)));
        let domain = Paragraph::new(domain)
            .block(
                Block::default()
                    .title(" Domain ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(from(COLOR_RED).unwrap_or(Color::Cyan))),
            )
            .alignment(Alignment::Left);

        domain.render(layout[0], &mut buffer);
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
            InsertPasswordState::Password => match key.code {
                KeyCode::Down | KeyCode::Tab | KeyCode::Enter | KeyCode::Up => {
                    self.state = InsertPasswordState::Quit;
                }
                KeyCode::Char('g') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.password = generate_password();
                        self.cursor = 0;
                        self.input_offset = 0;
                    } else {
                        let config = self.generate_input_config();
                        let (value, cursor_position, input_offset) =
                            Input::handle_key(key, &config, self.password());
                        self.password = value;
                        self.cursor = cursor_position;
                        self.input_offset = input_offset;
                    }
                }
                _ => {
                    let config = self.generate_input_config();
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.password());
                    self.password = value;
                    self.cursor = cursor_position;
                    self.input_offset = input_offset;
                }
            },
            InsertPasswordState::Quit => match key.code {
                KeyCode::Enter => {
                    app.mutable_app_state.popups.pop();
                    self.exit_state = Some(InsertPasswordExitState::Quit);
                    poped = true;
                }
                KeyCode::Up | KeyCode::Down => {
                    self.state = InsertPasswordState::Password;
                }
                KeyCode::Right | KeyCode::Tab | KeyCode::Left => {
                    self.state = InsertPasswordState::Confirm;
                }
                _ => {}
            },
            InsertPasswordState::Confirm => match key.code {
                KeyCode::Enter => {
                    app.mutable_app_state.popups.pop();
                    self.exit_state = Some(InsertPasswordExitState::Confirm);
                    poped = true;
                }
                KeyCode::Left | KeyCode::Right => {
                    self.state = InsertPasswordState::Quit;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Up => {
                    self.state = InsertPasswordState::Password;
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
        PopupType::InsertPassword
    }
}
