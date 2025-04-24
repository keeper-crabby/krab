use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
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

/// Represents the buttons in the insert master password popup
///
/// # Variants
/// * `Confirm` - The confirm button
/// * `Quit` - The quit button
#[derive(Debug, Clone, PartialEq)]
enum MasterPasswordButton {
    Confirm,
    Quit,
}

/// Represents the state of the insert master popup
///
/// # Variants
/// * `Master` - The master state
/// * `Confirm` - The confirm state
/// * `Quit` - The quit state
#[derive(Clone, PartialEq)]
pub enum InsertMasterState {
    Master,
    Confirm,
    Quit,
}

/// Represents the exit state of the insert master popup
///
/// # Variants
/// * `Confirm` - The confirm state
/// * `Quit` - The quit state
#[derive(Clone, PartialEq)]
pub enum InsertMasterExitState {
    Confirm,
    Quit,
}

/// Represents the insert master popup
///
/// # Fields
/// * `master` - The master
/// * `state` - The state
/// * `exit_state` - The exit state
/// * `cursors` - The cursors
/// * `input_offsets` - The input offsets
///
/// # Methods
/// * `new` - Creates a new `InsertMaster`
/// * `master` - Returns the master password
/// * `exit_state` - Returns the exit state
/// * `min_area` - Returns the minimum area of the popup
/// * `generate_input_config` - Returns the input config for the popup
/// * `generate_button_config` - Returns the button config for the popup
///
/// # Implements
/// * `Popup` - The popup trait
#[derive(Clone)]
pub struct InsertMaster {
    master: String,
    state: InsertMasterState,
    exit_state: Option<InsertMasterExitState>,
    cursor: u16,
    input_offset: u16,
}

impl InsertMaster {
    /// Creates a new insert master popup
    ///
    /// # Returns
    /// A new `InsertMaster`
    pub fn new() -> Self {
        let cursor = 0;
        let input_offset = 0;
        InsertMaster {
            master: String::new(),
            state: InsertMasterState::Master,
            exit_state: None,
            cursor,
            input_offset,
        }
    }

    /// Returns the master password
    ///
    /// # Returns
    /// The master password
    pub fn master(&self) -> String {
        self.master.clone()
    }

    /// Returns the state of the popup
    ///
    /// # Returns
    /// The state of the popup
    pub fn exit_state(&self) -> Option<InsertMasterExitState> {
        self.exit_state.clone()
    }

    /// Returns the minimum area of the popup
    /// 
    /// # Returns
    /// The minimum area of the popup
    pub fn min_area() -> (u16, u16) {
        (InputConfig::width(), InputConfig::height() + ButtonConfig::height())
    }

    /// Returns the maximum area of the popup
    /// 
    /// # Returns
    /// The maximum area of the popup
    fn generate_input_config(&self) -> InputConfig {
        InputConfig::new(
            self.state == InsertMasterState::Master,
            self.master(),
            true,
            "Master password".to_string(),
            if self.state == InsertMasterState::Master {
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
    fn generate_button_config(&self, input: MasterPasswordButton) -> ButtonConfig {
        match input {
            MasterPasswordButton::Confirm => ButtonConfig::new(
                self.state == InsertMasterState::Confirm,
                "Confirm".to_string(),
            ),
            MasterPasswordButton::Quit => {
                ButtonConfig::new(self.state == InsertMasterState::Quit, "Quit".to_string())
            }
        }
    }
}

impl Popup for InsertMaster {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        let height = InputConfig::height() + ButtonConfig::height();
        let width = InputConfig::width();
        let rect = centered_absolute_rect(rect, width, height);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(InputConfig::height()),
                Constraint::Length(ButtonConfig::height()),
            ])
            .split(rect);

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(layout[1]);

        let master_config = self.generate_input_config();

        let confirm_config = self.generate_button_config(MasterPasswordButton::Confirm);
        let quit_config = self.generate_button_config(MasterPasswordButton::Quit);
        f.render_widget(Clear, rect);
        let mut buffer = f.buffer_mut();

        Input::render(&mut buffer, layout[0], &master_config);
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
            InsertMasterState::Master => match key.code {
                KeyCode::Down | KeyCode::Tab | KeyCode::Enter | KeyCode::Up => {
                    self.state = InsertMasterState::Quit;
                }
                _ => {
                    let config = self.generate_input_config();
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, self.master());
                    self.master = value;
                    self.cursor = cursor_position;
                    self.input_offset = input_offset;
                }
            },
            InsertMasterState::Quit => match key.code {
                KeyCode::Enter => {
                    app.mutable_app_state.popups.pop();
                    self.exit_state = Some(InsertMasterExitState::Quit);
                    poped = true;
                }
                KeyCode::Up | KeyCode::Down => {
                    self.state = InsertMasterState::Master;
                }
                KeyCode::Right | KeyCode::Tab | KeyCode::Left => {
                    self.state = InsertMasterState::Confirm;
                }
                _ => {}
            },
            InsertMasterState::Confirm => match key.code {
                KeyCode::Enter => {
                    app.mutable_app_state.popups.pop();
                    self.exit_state = Some(InsertMasterExitState::Confirm);
                    poped = true;
                }
                KeyCode::Left | KeyCode::Right => {
                    self.state = InsertMasterState::Quit;
                }
                KeyCode::Down | KeyCode::Tab | KeyCode::Up => {
                    self.state = InsertMasterState::Master;
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
            InputConfig::height() + ButtonConfig::height(),
        )
    }

    fn popup_type(&self) -> PopupType {
        PopupType::InsertMaster
    }
}
