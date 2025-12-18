use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Clear},
    Frame,
};

use crate::{
    centered_absolute_rect,
    popups::{Popup, PopupType},
    Application,
};

/// Represents the exit popup
///
/// # Methods
/// * `new` - Creates a new `Exit`
/// * `min_area` - Returns the minimum area of the popup
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

    /// Returns the minimum area of the popup
    /// 
    /// # Returns
    /// The minimum area of the popup
    pub fn min_area() -> (u16, u16) {
        (30, 10)
    }
}

// Is this even necessary?

impl Popup for Exit {
    fn render(&self, f: &mut Frame, app: &Application, rect: Rect) {
        let theme = app.theme();
        let block = Block::default()
            .title(" Press q to exit ")
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.error()));
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
