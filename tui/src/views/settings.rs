use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use krab_backend::Config;

use crate::{
    from,
    views::{startup::StartUp, View, ViewState},
    Application, COLOR_ORANGE, COLOR_WHITE,
};

/// Represents the settings options
///
/// # Variants
/// * `IncludeNumbers` - Include numbers in password generation
/// * `IncludeSpecialChars` - Include special characters in password generation
/// * `Save` - Save the current settings
/// * `Back` - Go back to startup
#[derive(Debug, Clone, PartialEq)]
enum SettingsOption {
    IncludeNumbers,
    IncludeSpecialChars,
    Save,
    Back,
}

/// Represents the settings view
///
/// # Fields
/// * `selected_option` - The currently selected option
/// * `config` - The password configuration to modify
/// * `original_config` - The original configuration to compare against for unsaved changes
/// * `has_unsaved_changes` - Whether there are unsaved changes
///
/// # Methods
/// * `new` - Creates a new `Settings` view
///
/// # Implements
/// * `View` - The view trait
#[derive(Clone)]
pub struct Settings {
    selected_option: SettingsOption,
    config: Config,
    original_config: Config,
    has_unsaved_changes: bool,
}

impl Settings {
    /// Creates a new `Settings`
    ///
    /// # Returns
    /// A new `Settings`
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        Settings {
            selected_option: SettingsOption::IncludeNumbers,
            config: config.clone(),
            original_config: config.clone(),
            has_unsaved_changes: false,
        }
    }

    /// Updates the unsaved changes flag by comparing current config with original
    fn update_unsaved_changes(&mut self) {
        self.has_unsaved_changes = self.config != self.original_config;
    }

    /// Gets the list of settings items for rendering
    fn get_settings_items(&self) -> Vec<ListItem> {
        vec![
            ListItem::new(Line::from(vec![Span::styled(
                format!(
                    "[{}] Include Numbers",
                    if self.config.password_config.include_numbers {
                        "x"
                    } else {
                        " "
                    }
                ),
                if self.selected_option == SettingsOption::IncludeNumbers {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                },
            )])),
            ListItem::new(Line::from(vec![Span::styled(
                format!(
                    "[{}] Include Special Characters",
                    if self.config.password_config.include_special {
                        "x"
                    } else {
                        " "
                    }
                ),
                if self.selected_option == SettingsOption::IncludeSpecialChars {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                },
            )])),
            ListItem::new(Line::from(vec![Span::styled(
                "Save Settings",
                if self.selected_option == SettingsOption::Save {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                },
            )])),
            ListItem::new(Line::from(vec![Span::styled(
                "< Back",
                if self.selected_option == SettingsOption::Back {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                },
            )])),
        ]
    }
}

impl View for Settings {
    fn render(&self, f: &mut Frame, _app: &Application, rect: Rect) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Settings list
                Constraint::Length(3), // Instructions
            ])
            .split(rect);

        // Title with unsaved changes indicator
        let title_text = if self.has_unsaved_changes {
            "Settings *"
        } else {
            "Settings"
        };
        let title = Paragraph::new(title_text)
            .style(Style::default().fg(from(COLOR_ORANGE).unwrap_or(Color::Yellow)))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Settings list
        let settings_items = self.get_settings_items();
        let settings_list = List::new(settings_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Password Generation"),
            )
            .style(Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::Yellow)));

        f.render_widget(settings_list, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(
            "j/k - navigate | Space/Enter - toggle/select | * = unsaved changes | q/Esc - back",
        )
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[2]);
    }

    fn handle_key(&mut self, key: &KeyEvent, app: &Application) -> Application {
        let mut app = app.clone();
        let mut change_state = false;

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_option = match self.selected_option {
                    SettingsOption::IncludeNumbers => SettingsOption::IncludeSpecialChars,
                    SettingsOption::IncludeSpecialChars => SettingsOption::Save,
                    SettingsOption::Save => SettingsOption::Back,
                    SettingsOption::Back => SettingsOption::IncludeNumbers,
                };
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_option = match self.selected_option {
                    SettingsOption::IncludeNumbers => SettingsOption::Back,
                    SettingsOption::IncludeSpecialChars => SettingsOption::IncludeNumbers,
                    SettingsOption::Save => SettingsOption::IncludeSpecialChars,
                    SettingsOption::Back => SettingsOption::Save,
                };
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.selected_option {
                SettingsOption::IncludeNumbers => {
                    self.config.password_config.include_numbers =
                        !self.config.password_config.include_numbers;
                    self.update_unsaved_changes();
                }
                SettingsOption::IncludeSpecialChars => {
                    self.config.password_config.include_special =
                        !self.config.password_config.include_special;
                    self.update_unsaved_changes();
                }
                SettingsOption::Save => {
                    // Save the password config and update original config
                    if let Ok(()) = self.config.save() {
                        self.original_config = self.config.clone();
                        self.has_unsaved_changes = false;
                    }
                }
                SettingsOption::Back => {
                    // Go back without saving
                    app.state = ViewState::StartUp(StartUp::new());
                    change_state = true;
                }
            },
            KeyCode::Esc | KeyCode::Char('q') => {
                // Go back without saving
                app.state = ViewState::StartUp(StartUp::new());
                change_state = true;
            }
            _ => {}
        }

        if !change_state {
            app.state = ViewState::Settings(self.clone());
        }

        app
    }

    fn min_area(&self) -> (u16, u16) {
        (60, 15)
    }

    fn needs_header(&self) -> bool {
        false
    }
}
