use ratatui::{
    crossterm::event::KeyEvent,
    prelude::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Padding, Paragraph},
    Frame,
};

use crate::{
    centered_absolute_rect, from,
    popups::{Popup, PopupType},
    Application, COLOR_RED,
};

/// Represents the message popup
///
/// # Fields
/// * `message` - The message
///
/// # Methods
/// * `new` - Creates a new `MessagePopup`
///
/// # Implements
/// * `Popup` - The popup trait
#[derive(Clone)]
pub struct MessagePopup {
    message: String,
}

impl MessagePopup {
    /// Creates a new message popup
    ///
    /// # Arguments
    /// * `message` - The message
    ///
    /// # Returns
    /// A new `MessagePopup`
    pub fn new(message: String) -> Self {
        MessagePopup { message }
    }
}

impl Popup for MessagePopup {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        let message_p = Paragraph::new(self.message.clone())
            .style(Style::default().fg(from(COLOR_RED).unwrap_or(Color::Red)))
            .block(
                Block::bordered()
                    .title(" Press any key to continue ")
                    .padding(Padding::new(0, 0, rect.height / 3, 0))
                    .border_style(Style::default().fg(from(COLOR_RED).unwrap_or(Color::White))),
            )
            .alignment(Alignment::Center);

        f.render_widget(Clear, rect);
        f.render_widget(message_p, rect);
    }

    fn handle_key(
        &mut self,
        _key: &KeyEvent,
        app: &Application,
    ) -> (Application, Option<Box<dyn Popup>>) {
        let mut app = app.clone();
        app.mutable_app_state.popups.pop();

        (app, None)
    }

    fn wrapper(&self, rect: Rect) -> Rect {
        centered_absolute_rect(rect, 30, 10)
    }

    fn popup_type(&self) -> PopupType {
        PopupType::Message
    }
}
