use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::Widget,
};

/// Represents the configuration of a window too small
///
/// # Fields
/// * `min_width` - The minimum width
/// * `min_height` - The minimum height
///
/// # Methods
/// * `new` - Creates a new `WindowTooSmallConfig`
pub struct WindowTooSmallConfig {
    min_width: u16,
    min_height: u16,
}

/// Represents a window too small
///
/// # Methods
/// * `render` - Renders the window too small
pub struct WindowTooSmall {}

impl WindowTooSmallConfig {
    /// Creates a new `WindowTooSmallConfig`
    ///
    /// # Arguments
    /// * `min_width` - The minimum width
    /// * `min_height` - The minimum height
    ///
    /// # Returns
    /// A new `WindowTooSmallConfig`
    pub fn new(min_width: u16, min_height: u16) -> Self {
        Self {
            min_width,
            min_height,
        }
    }
}

impl WindowTooSmall {
    /// Renders the window too small
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `rect` - The rectangle to render the window too small in
    /// * `config` - The configuration of the window too small
    pub fn render(buffer: &mut Buffer, rect: Rect, config: &WindowTooSmallConfig) {
        let text = format!(
            "The window is too small. The minimum size is {}x{}.",
            config.min_width, config.min_height
        );
        let text = Line::from(text);

        text.render(rect, buffer);
    }
}
