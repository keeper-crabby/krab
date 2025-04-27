use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::{from, COLOR_ORANGE, COLOR_WHITE};

const INPUT_HEIGHT: u16 = 3;
const DEFAULT_INPUT_WIDTH: u16 = 32;
const MAX_INPUT_LENGTH: u16 = 256;
const PADDING: u16 = 2;

/// Represents the configuration of an input
///
/// # Fields
/// * `focused` - Indicates if the input is focused
/// * `value` - The value of the input
/// * `hidden` - Indicates if the input is hidden
/// * `title` - The title of the input
/// * `cursor_position` - The cursor position of the input
/// * `input_offset` - The input offset of the input
/// * `width` - The width of the input
///
/// # Methods
/// * `new` - Creates a new `InputConfig`
/// * `height` - Returns the height of the input
/// * `default_width` - Returns the default width of the input
/// * `width` - Returns the width of the input
pub struct InputConfig {
    focused: bool,
    value: String,
    hidden: bool,
    title: String,
    cursor_position: Option<u16>,
    input_offset: u16,
    width: Option<u16>,
}

/// Represents an input
///
/// # Methods
/// * `render` - Renders the input
/// * `handle_key` - Handles a key event
pub struct Input {}

impl InputConfig {
    /// Creates a new `InputConfig`
    ///
    /// # Arguments
    /// * `focused` - Indicates if the input is focused
    /// * `value` - The value of the input
    /// * `hidden` - Indicates if the input is hidden
    /// * `title` - The title of the input
    /// * `cursor_position` - The cursor position of the input
    /// * `input_offset` - The input offset of the input
    ///
    /// # Returns
    /// A new `InputConfig`
    pub fn new(
        focused: bool,
        value: String,
        hidden: bool,
        title: String,
        cursor_position: Option<u16>,
        input_offset: u16,
        width: Option<u16>,
    ) -> Self {
        Self {
            focused,
            value,
            hidden,
            title,
            cursor_position,
            input_offset,
            width,
        }
    }

    /// Returns the height of the input
    ///
    /// # Returns
    /// The height of the input
    pub fn height() -> u16 {
        INPUT_HEIGHT
    }

    /// Returns the width of the input
    ///
    /// # Returns
    /// The width of the input
    pub fn default_width() -> u16 {
        DEFAULT_INPUT_WIDTH + 2 * PADDING
    }

    /// Returns the maximum length of the input
    ///
    /// # Returns
    /// The maximum length of the input
    pub fn width(&self) -> u16 {
        self.width.unwrap_or(DEFAULT_INPUT_WIDTH) + 2 * PADDING
    }
}

impl Input {
    /// Renders the input
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `rect` - The rectangle to render the input in
    /// * `config` - The configuration of the input
    pub fn render(buffer: &mut Buffer, rect: Rect, config: &InputConfig) {
        assert!(rect.height >= INPUT_HEIGHT);
        assert!(config.value.len() as u16 <= MAX_INPUT_LENGTH);
        assert_eq!(config.focused ^ config.cursor_position.is_some(), false);

        let rect = Rect::new(rect.x, rect.y, rect.width, INPUT_HEIGHT);

        let mut text = if config.hidden {
            let mut hidden_text = String::new();
            for _ in 0..config.value.len() {
                hidden_text.push('*');
            }
            hidden_text
        } else {
            config.value.clone()
        };

        let mut text = text.split_off(config.input_offset as usize);
        text.truncate(config.width.unwrap_or(DEFAULT_INPUT_WIDTH) as usize);

        let text = if config.focused {
            let cursor_position =
                config.cursor_position.unwrap_or(0) as usize - config.input_offset as usize;
            let mut first_part = text.clone();
            let mut second_part = first_part.split_off(cursor_position);

            if second_part.len() > 0 {
                second_part = second_part.split_off(1);
            }

            let first_part_len = first_part.len() as u16;
            let second_part_len = second_part.len() as u16;

            let mut line = vec![
                Span::raw(first_part)
                    .style(Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White))),
                Span::raw("█")
                    .style(Style::default().fg(from(COLOR_ORANGE).unwrap_or(Color::Yellow))),
                Span::raw(second_part)
                    .style(Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White))),
            ];

            if first_part_len + second_part_len < config.width.unwrap_or(DEFAULT_INPUT_WIDTH) {
                line.push(Span::raw(" ".repeat(
                    (config.width.unwrap_or(DEFAULT_INPUT_WIDTH) - first_part_len - second_part_len)
                        as usize,
                )));
            }

            let text = Line::from(line).centered();

            text
        } else {
            let text_len = text.len() as u16;

            Line::from(vec![
                Span::raw(text)
                    .style(Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White))),
                Span::raw(
                    " ".repeat((config.width.unwrap_or(DEFAULT_INPUT_WIDTH) - text_len) as usize),
                ),
            ])
            .centered()
        };

        let paragraph = Paragraph::new(text).block(
            Block::bordered()
                .border_style(Style::default().fg(if config.focused {
                    from(COLOR_ORANGE).unwrap_or(Color::Yellow)
                } else {
                    from(COLOR_WHITE).unwrap_or(Color::White)
                }))
                .title(" ".to_string() + &config.title + " ")
                .title_style(Style::default().fg(if config.focused {
                    from(COLOR_ORANGE).unwrap_or(Color::Yellow)
                } else {
                    from(COLOR_WHITE).unwrap_or(Color::White)
                })),
        );

        paragraph.render(rect, buffer);
    }

    /// Handles a key event
    ///
    /// # Arguments
    /// * `key` - The key event to handle
    /// * `config` - The configuration of the input
    /// * `previous_value` - The previous value of the input
    ///
    /// # Returns
    /// A tuple containing the new value and the new cursor position
    pub fn handle_key(
        key: &KeyEvent,
        config: &InputConfig,
        previous_value: &str,
    ) -> (String, u16, u16) {
        let mut value = previous_value.to_string();
        let mut cursor_position = config.cursor_position.unwrap_or(value.len() as u16);
        let previous_cursor_position = config.cursor_position.unwrap_or(0);
        let mut input_offset = config.input_offset;

        match key.code {
            KeyCode::Char(c) => {
                if value.len() as u16 == MAX_INPUT_LENGTH {
                    return (value, cursor_position, input_offset);
                }
                value.insert(cursor_position as usize, c);
                cursor_position += 1;
            }
            KeyCode::Backspace => {
                if cursor_position > 0 {
                    value.remove(cursor_position as usize - 1);
                    cursor_position -= 1;
                }
            }
            KeyCode::Delete => {
                if cursor_position < value.len() as u16 {
                    value.remove(cursor_position as usize);
                }
                if cursor_position > value.len() as u16 {
                    cursor_position = value.len() as u16;
                }
            }
            KeyCode::Left => {
                if cursor_position > 0 {
                    cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if cursor_position == MAX_INPUT_LENGTH - 1 {
                    return (value, cursor_position, input_offset);
                }
                if cursor_position < value.len() as u16 {
                    cursor_position += 1;
                }
            }
            _ => {}
        }

        if cursor_position != 0
            && previous_cursor_position == cursor_position - 1
            && cursor_position == input_offset + config.width.unwrap_or(DEFAULT_INPUT_WIDTH)
        {
            input_offset = cursor_position - config.width.unwrap_or(DEFAULT_INPUT_WIDTH) + 1;
        } else if previous_cursor_position == cursor_position + 1 && cursor_position <= input_offset
        {
            if input_offset != 0 {
                input_offset = input_offset - 1;
            }
        }

        (value, cursor_position, input_offset)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::KeyModifiers;

    use super::*;

    #[test]
    fn test_input_insert() {
        let config = InputConfig::new(
            true,
            "test".to_string(),
            false,
            "Test".to_string(),
            Some(4),
            0,
            None,
        );
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let (value, cursor_position, input_offset) = Input::handle_key(&key, &config, "test");
        assert_eq!(value, "testa".to_string());
        assert_eq!(cursor_position, 5);
        assert_eq!(input_offset, 0);
    }

    #[test]
    fn test_input_insert_max() {
        let mut value = String::new();
        for _ in 0..MAX_INPUT_LENGTH {
            value.push('a');
        }
        let config = InputConfig::new(
            true,
            value.clone(),
            false,
            "Test".to_string(),
            Some(3),
            0,
            None,
        );
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let (value, cursor_position, input_offset) =
            Input::handle_key(&key, &config, value.as_str());
        assert_eq!(value, value.clone());
        assert_eq!(cursor_position, 3);
        assert_eq!(input_offset, 0);
    }

    #[test]
    fn test_input_backspace() {
        let config = InputConfig::new(
            true,
            "test".to_string(),
            false,
            "Test".to_string(),
            Some(4),
            0,
            None,
        );
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        let (value, cursor_position, input_offset) = Input::handle_key(&key, &config, "test");
        assert_eq!(value, "tes".to_string());
        assert_eq!(cursor_position, 3);
        assert_eq!(input_offset, 0);
    }

    #[test]
    fn test_input_delete() {
        let config = InputConfig::new(
            true,
            "test".to_string(),
            false,
            "Test".to_string(),
            Some(1),
            0,
            None,
        );
        let key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        let (value, cursor_position, input_offset) = Input::handle_key(&key, &config, "test");
        assert_eq!(value, "tst".to_string());
        assert_eq!(cursor_position, 1);
        assert_eq!(input_offset, 0);
    }

    #[test]
    fn test_input_left() {
        let config = InputConfig::new(
            true,
            "test".to_string(),
            false,
            "Test".to_string(),
            Some(2),
            0,
            None,
        );
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let (value, cursor_position, input_offset) = Input::handle_key(&key, &config, "test");
        assert_eq!(value, "test".to_string());
        assert_eq!(cursor_position, 1);
        assert_eq!(input_offset, 0);
    }

    #[test]
    fn test_input_right() {
        let config = InputConfig::new(
            true,
            "test".to_string(),
            false,
            "Test".to_string(),
            Some(2),
            0,
            None,
        );
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let (value, cursor_position, input_offset) = Input::handle_key(&key, &config, "test");
        assert_eq!(value, "test".to_string());
        assert_eq!(cursor_position, 3);
        assert_eq!(input_offset, 0);
    }

    #[test]
    fn test_input_right_max() {
        let mut value = String::new();
        for _ in 0..MAX_INPUT_LENGTH {
            value.push('a');
        }
        let config = InputConfig::new(
            true,
            value.clone(),
            false,
            "Test".to_string(),
            Some(255),
            255 - DEFAULT_INPUT_WIDTH + 1,
            None,
        );
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let (value, cursor_position, input_offset) =
            Input::handle_key(&key, &config, value.as_str());
        assert_eq!(value, value.clone());
        assert_eq!(cursor_position, 255);
        assert_eq!(input_offset, 255 - DEFAULT_INPUT_WIDTH + 1);
    }

    #[test]
    fn test_input_first_offset() {
        let config = InputConfig::new(
            true,
            "0123456789012345678901234567890".to_string(),
            false,
            "Test".to_string(),
            Some(31),
            0,
            None,
        );
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        let (value, cursor_position, input_offset) =
            Input::handle_key(&key, &config, "0123456789012345678901234567890");
        assert_eq!(value, "01234567890123456789012345678901".to_string());
        assert_eq!(cursor_position, 32);
        assert_eq!(input_offset, 1);
    }

    #[test]
    fn test_input_first_offset_left() {
        let config = InputConfig::new(
            true,
            "0123456789012345678901234567890123456789012345678901234567890".to_string(),
            false,
            "Test".to_string(),
            Some(29),
            28,
            None,
        );
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        let (value, cursor_position, input_offset) = Input::handle_key(
            &key,
            &config,
            "0123456789012345678901234567890123456789012345678901234567890",
        );
        assert_eq!(
            value,
            "012345678901234567890123456790123456789012345678901234567890".to_string()
        );
        assert_eq!(cursor_position, 28);
        assert_eq!(input_offset, 27);
    }
}
