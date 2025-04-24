use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
    Frame,
};

use crate::{
    centered_absolute_rect, from,
    popups::{Popup, PopupType},
    Application, COLOR_RED,
};

/// Represents the exit popup
///
/// # Methods
/// * `new` - Creates a new `Exit`
///
/// # Implements
/// * `Popup` - The popup trait
#[derive(Clone)]
pub struct Exit {}

impl Exit {
    /// Creates a new exit popup
    ///
    /// # Returns
    /// A new `Exit`
    pub fn new() -> Self {
        Exit {}
    }
}

// Is this even necessary?

impl Popup for Exit {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        let block = Block::default()
            .title(" Press q to exit ")
            .borders(Borders::ALL)
            .style(Style::default().fg(from(COLOR_RED).unwrap_or(Color::Red)));
        f.render_widget(Clear, rect);
        f.render_widget(block, rect);
    }

    fn handle_key(
        &mut self,
        key: &KeyEvent,
        app: &Application,
    ) -> (Application, Option<Box<dyn Popup>>) {
        let mut app = app.clone();
        match key.code {
            KeyCode::Char('q') => {
                app.mutable_app_state.running = false;
            }
            _ => {}
        }

        (app, None)
    }

    fn wrapper(&self, rect: Rect) -> Rect {
        centered_absolute_rect(rect, 30, 10)
    }

    fn popup_type(&self) -> PopupType {
        PopupType::Exit
    }
}
