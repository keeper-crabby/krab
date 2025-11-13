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
/// * `width` - The width of the popup
/// * `height` - The height of the popup
///
/// # Methods
/// * `new` - Creates a new `MessagePopup`
/// * `new_with_size` - Creates a new `MessagePopup` with custom dimensions
/// * `min_area` - Returns the minimum area of the popup
///
/// # Implements
/// * `Popup` - The popup trait
#[derive(Clone)]
pub struct MessagePopup {
    message: String,
    width: u16,
    height: u16,
}

impl MessagePopup {
    /// Creates a new message popup with default size
    ///
    /// # Arguments
    /// * `message` - The message
    ///
    /// # Returns
    /// A new `MessagePopup` with default size (30x10)
    pub fn new(message: String) -> Self {
        MessagePopup {
            message,
            width: 30,
            height: 10,
        }
    }

    /// Creates a new message popup with custom size
    ///
    /// # Arguments
    /// * `message` - The message
    /// * `width` - The width of the popup
    /// * `height` - The height of the popup
    ///
    /// # Returns
    /// A new `MessagePopup` with custom dimensions
    pub fn new_with_size(message: String, width: u16, height: u16) -> Self {
        MessagePopup {
            message,
            width,
            height,
        }
    }

    /// Returns the minimum area of the popup
    ///
    /// # Returns
    /// The minimum area of the popup
    pub fn min_area() -> (u16, u16) {
        (30, 10)
    }
}

impl Popup for MessagePopup {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        // Count lines in the message to determine if we need special formatting
        let line_count = self.message.lines().count() as u16;

        // For multi-line messages (like help text), use minimal padding and left alignment
        // For short messages, center them vertically
        let (padding, alignment) = if line_count > 3 {
            (Padding::new(2, 2, 1, 1), Alignment::Left)
        } else {
            (Padding::new(0, 0, rect.height / 3, 0), Alignment::Center)
        };

        let message_p = Paragraph::new(self.message.clone())
            .style(Style::default().fg(from(COLOR_RED).unwrap_or(Color::Red)))
            .block(
                Block::bordered()
                    .title(" Press any key to continue ")
                    .padding(padding)
                    .border_style(Style::default().fg(from(COLOR_RED).unwrap_or(Color::White))),
            )
            .alignment(alignment);

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
        centered_absolute_rect(rect, self.width, self.height)
    }

    fn popup_type(&self) -> PopupType {
        PopupType::Message
    }
}
