use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::{Alignment, Rect},
    style::Style,
    widgets::{Block, Clear, Padding, Paragraph},
    Frame,
};

use crate::{
    centered_absolute_rect,
    popups::{Popup, PopupType},
    Application,
};

/// Represents the exit state of the confirm delete user popup
///
/// # Variants
/// * `Confirm` - The user confirmed deletion
/// * `Quit` - The user cancelled
#[derive(Clone, PartialEq)]
pub enum ConfirmDeleteUserExitState {
    Confirm,
    Quit,
}

/// Represents the confirm delete user popup
///
/// # Fields
/// * `exit_state` - The exit state
///
/// # Methods
/// * `new` - Creates a new `ConfirmDeleteUser`
/// * `exit_state` - Returns the exit state
/// * `min_area` - Returns the minimum area of the popup
///
/// # Implements
/// * `Popup` - The popup trait
#[derive(Clone)]
pub struct ConfirmDeleteUser {
    exit_state: Option<ConfirmDeleteUserExitState>,
}

impl ConfirmDeleteUser {
    /// Creates a new confirm delete user popup
    ///
    /// # Returns
    /// A new `ConfirmDeleteUser`
    pub fn new() -> Self {
        ConfirmDeleteUser { exit_state: None }
    }

    /// Returns the exit state of the popup
    ///
    /// # Returns
    /// The exit state of the popup
    pub fn exit_state(&self) -> Option<ConfirmDeleteUserExitState> {
        self.exit_state.clone()
    }

    /// Returns the minimum area of the popup
    ///
    /// # Returns
    /// The minimum area of the popup
    pub fn min_area() -> (u16, u16) {
        (40, 10)
    }
}

impl Popup for ConfirmDeleteUser {
    fn render(&self, f: &mut Frame, app: &Application, rect: Rect) {
        let theme = app.theme();
        let message = "Delete account and all\npasswords? This cannot\nbe undone.\n\nPress Y to confirm\nPress N to cancel";

        let message_p = Paragraph::new(message)
            .style(Style::default().fg(theme.error()))
            .block(
                Block::bordered()
                    .title(" Delete Account ")
                    .padding(Padding::new(2, 2, 1, 1))
                    .border_style(Style::default().fg(theme.error())),
            )
            .alignment(Alignment::Center);

        f.render_widget(Clear, rect);
        f.render_widget(message_p, rect);
    }

    fn handle_key(
        &mut self,
        key: &KeyEvent,
        app: &Application,
    ) -> (Application, Option<Box<dyn Popup>>) {
        let mut app = app.clone();

        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.mutable_app_state.popups.pop();
                self.exit_state = Some(ConfirmDeleteUserExitState::Confirm);
                (app, Some(Box::new(self.clone())))
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.mutable_app_state.popups.pop();
                self.exit_state = Some(ConfirmDeleteUserExitState::Quit);
                (app, Some(Box::new(self.clone())))
            }
            _ => {
                // Ignore other keys, keep popup open
                (app, None)
            }
        }
    }

    fn wrapper(&self, rect: Rect) -> Rect {
        centered_absolute_rect(rect, 40, 10)
    }

    fn popup_type(&self) -> PopupType {
        PopupType::ConfirmDeleteUser
    }
}
