use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use krab_backend::Config;

use crate::{
    theme::{BuiltinTheme, ThemeConfig},
    views::{startup::StartUp, View, ViewState},
    Application,
};

/// Represents the settings options
///
/// # Variants
/// * `IncludeNumbers` - Include numbers in password generation
/// * `IncludeSpecialChars` - Include special characters in password generation
/// * `ThemeSelection` - Select the theme
/// * `Save` - Save the current settings
/// * `Back` - Go back to startup
#[derive(Debug, Clone, PartialEq)]
enum SettingsOption {
    IncludeNumbers,
    IncludeSpecialChars,
    ThemeSelection,
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
/// * `selected_theme_index` - The index of the currently selected theme
/// * `available_themes` - The list of available built-in themes
/// * `original_theme_index` - The original theme index for detecting changes
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
    selected_theme_index: usize,
    available_themes: Vec<BuiltinTheme>,
    original_theme_index: usize,
}

impl Settings {
    /// Creates a new `Settings`
    ///
    /// # Returns
    /// A new `Settings`
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        let available_themes = BuiltinTheme::all();

        // Determine current theme index from config
        let current_theme_config: ThemeConfig = config
            .theme
            .clone()
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let selected_theme_index = match current_theme_config {
            ThemeConfig::Builtin(builtin) => available_themes
                .iter()
                .position(|t| *t == builtin)
                .unwrap_or(0),
            ThemeConfig::Custom(_) => 0, // Default to first for custom
        };

        Settings {
            selected_option: SettingsOption::IncludeNumbers,
            config: config.clone(),
            original_config: config.clone(),
            has_unsaved_changes: false,
            selected_theme_index,
            available_themes,
            original_theme_index: selected_theme_index,
        }
    }

    /// Updates the unsaved changes flag by comparing current config with original
    fn update_unsaved_changes(&mut self) {
        self.has_unsaved_changes = self.config != self.original_config
            || self.selected_theme_index != self.original_theme_index;
    }

    /// Gets the list of settings items for rendering
    fn get_settings_items(&self, app: &Application) -> Vec<ListItem> {
        let theme = app.theme();
        let current_theme_name = self.available_themes[self.selected_theme_index].name();

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
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
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
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
                },
            )])),
            ListItem::new(Line::from(vec![Span::styled(
                format!("Theme: < {} >", current_theme_name),
                if self.selected_option == SettingsOption::ThemeSelection {
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
                },
            )])),
            ListItem::new(Line::from(vec![Span::styled(
                "Save Settings",
                if self.selected_option == SettingsOption::Save {
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
                },
            )])),
            ListItem::new(Line::from(vec![Span::styled(
                "< Back",
                if self.selected_option == SettingsOption::Back {
                    Style::default().fg(theme.accent())
                } else {
                    Style::default().fg(theme.fg())
                },
            )])),
        ]
    }
}

impl View for Settings {
    fn render(&self, f: &mut Frame, app: &Application, rect: Rect) {
        let theme = app.theme();

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
            .style(Style::default().fg(theme.accent()))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Settings list
        let settings_items = self.get_settings_items(app);
        let settings_list = List::new(settings_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Settings"),
            )
            .style(Style::default().fg(theme.fg()));

        f.render_widget(settings_list, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(
            "j/k - navigate | Space/Enter - toggle/select | h/l - change theme | * = unsaved | q/Esc - back",
        )
        .style(Style::default().fg(theme.fg()))
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
                    SettingsOption::IncludeSpecialChars => SettingsOption::ThemeSelection,
                    SettingsOption::ThemeSelection => SettingsOption::Save,
                    SettingsOption::Save => SettingsOption::Back,
                    SettingsOption::Back => SettingsOption::IncludeNumbers,
                };
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_option = match self.selected_option {
                    SettingsOption::IncludeNumbers => SettingsOption::Back,
                    SettingsOption::IncludeSpecialChars => SettingsOption::IncludeNumbers,
                    SettingsOption::ThemeSelection => SettingsOption::IncludeSpecialChars,
                    SettingsOption::Save => SettingsOption::ThemeSelection,
                    SettingsOption::Back => SettingsOption::Save,
                };
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected_option == SettingsOption::ThemeSelection {
                    if self.selected_theme_index > 0 {
                        self.selected_theme_index -= 1;
                    } else {
                        self.selected_theme_index = self.available_themes.len() - 1;
                    }
                    self.update_unsaved_changes();
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.selected_option == SettingsOption::ThemeSelection {
                    self.selected_theme_index =
                        (self.selected_theme_index + 1) % self.available_themes.len();
                    self.update_unsaved_changes();
                }
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
                SettingsOption::ThemeSelection => {
                    // Cycle through themes on Enter/Space as well
                    self.selected_theme_index =
                        (self.selected_theme_index + 1) % self.available_themes.len();
                    self.update_unsaved_changes();
                }
                SettingsOption::Save => {
                    // Save the theme config
                    let theme_config =
                        ThemeConfig::Builtin(self.available_themes[self.selected_theme_index]);
                    self.config.theme = Some(serde_json::to_value(&theme_config).unwrap());

                    // Save the config and update original config
                    if let Ok(()) = self.config.save() {
                        // Update the live theme in Application
                        app.set_theme(theme_config.resolve());
                        self.original_config = self.config.clone();
                        self.original_theme_index = self.selected_theme_index;
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
