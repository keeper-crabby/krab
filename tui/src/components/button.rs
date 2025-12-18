use ratatui::{
    prelude::{Buffer, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::theme::Theme;

const BUTTON_HEIGHT: u16 = 3;

/// Represents the configuration of a button
///
/// # Fields
/// * `focused` - Indicates if the button is focused
/// * `title` - The title of the button
/// * `theme` - The theme to use for styling
///
/// # Methods
///
/// * `new` - Creates a new `ButtonConfig`
/// * `height` - Returns the height of the button
pub struct ButtonConfig<'a> {
    focused: bool,
    title: String,
    theme: &'a Theme,
}

/// Represents a button
///
/// # Methods
/// * `render` - Renders the button
pub struct Button {}

impl<'a> ButtonConfig<'a> {
    /// Creates a new `ButtonConfig`
    ///
    /// # Arguments
    /// * `focused` - Indicates if the button is focused
    /// * `title` - The title of the button
    /// * `theme` - The theme to use for styling
    ///
    /// # Returns
    /// A new `ButtonConfig`
    pub fn new(focused: bool, title: String, theme: &'a Theme) -> Self {
        Self { focused, title, theme }
    }

    /// Returns the height of the button
    ///
    /// # Returns
    /// The height of the button
    pub fn height() -> u16 {
        BUTTON_HEIGHT
    }
}

impl Button {
    /// Renders the button
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `rect` - The rectangle to render the button in
    /// * `config` - The configuration of the button
    pub fn render(buffer: &mut Buffer, rect: Rect, config: &ButtonConfig) {
        assert!(rect.height >= BUTTON_HEIGHT);

        let rect = Rect::new(rect.x, rect.y, rect.width, BUTTON_HEIGHT);

        let text = config.title.clone();
        let text = Line::from(text)
            .style(
                Style::default()
                    .fg(config.theme.accent())
                    .add_modifier(if config.focused {
                        Modifier::ITALIC
                    } else {
                        Modifier::empty()
                    }),
            )
            .centered();

        let paragraph = Paragraph::new(text).block(Block::bordered().border_style(
            Style::default().fg(if config.focused {
                config.theme.accent()
            } else {
                config.theme.fg()
            }),
        ));

        paragraph.render(rect, buffer);
    }
}
