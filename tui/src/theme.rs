use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Represents a color theme for the application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    /// Primary foreground color (text, unfocused borders)
    pub foreground: String,
    /// Accent color for focused elements
    pub accent: String,
    /// Error/warning color
    pub error: String,
    /// Contrast/selection background color
    pub contrast: String,
}

impl Theme {
    /// Converts the theme's foreground hex to a ratatui Color
    pub fn fg(&self) -> Color {
        hex_to_color(&self.foreground).unwrap_or(Color::White)
    }

    /// Converts the theme's accent hex to a ratatui Color
    pub fn accent(&self) -> Color {
        hex_to_color(&self.accent).unwrap_or(Color::Yellow)
    }

    /// Converts the theme's error hex to a ratatui Color
    pub fn error(&self) -> Color {
        hex_to_color(&self.error).unwrap_or(Color::Red)
    }

    /// Converts the theme's contrast hex to a ratatui Color
    pub fn contrast(&self) -> Color {
        hex_to_color(&self.contrast).unwrap_or(Color::Black)
    }
}

/// Convert hex string to ratatui Color
fn hex_to_color(hex: &str) -> Result<Color, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Invalid hex color length".to_string());
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component")?;
    Ok(Color::Rgb(r, g, b))
}

/// Available built-in themes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BuiltinTheme {
    Warm,
    Dark,
    Light,
    Ocean,
    Forest,
}

impl BuiltinTheme {
    /// Returns all available built-in themes
    pub fn all() -> Vec<BuiltinTheme> {
        vec![
            BuiltinTheme::Warm,
            BuiltinTheme::Dark,
            BuiltinTheme::Light,
            BuiltinTheme::Ocean,
            BuiltinTheme::Forest,
        ]
    }

    /// Returns the display name of the theme
    pub fn name(&self) -> &'static str {
        match self {
            BuiltinTheme::Warm => "Warm (Default)",
            BuiltinTheme::Dark => "Dark",
            BuiltinTheme::Light => "Light",
            BuiltinTheme::Ocean => "Ocean",
            BuiltinTheme::Forest => "Forest",
        }
    }

    /// Returns the Theme data for this preset
    pub fn theme(&self) -> Theme {
        match self {
            BuiltinTheme::Warm => Theme {
                foreground: "#F0ECC9".to_string(),
                accent: "#E3AD43".to_string(),
                error: "#D44C1A".to_string(),
                contrast: "#503D2D".to_string(),
            },
            BuiltinTheme::Dark => Theme {
                foreground: "#C0C0C0".to_string(),
                accent: "#61AFEF".to_string(),
                error: "#E06C75".to_string(),
                contrast: "#282C34".to_string(),
            },
            BuiltinTheme::Light => Theme {
                foreground: "#383A42".to_string(),
                accent: "#4078F2".to_string(),
                error: "#E45649".to_string(),
                contrast: "#FAFAFA".to_string(),
            },
            BuiltinTheme::Ocean => Theme {
                foreground: "#D8DEE9".to_string(),
                accent: "#88C0D0".to_string(),
                error: "#BF616A".to_string(),
                contrast: "#2E3440".to_string(),
            },
            BuiltinTheme::Forest => Theme {
                foreground: "#D3C6AA".to_string(),
                accent: "#A7C080".to_string(),
                error: "#E67E80".to_string(),
                contrast: "#2D353B".to_string(),
            },
        }
    }
}

/// Theme configuration that can be saved/loaded
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeConfig {
    /// Use a built-in theme preset
    Builtin(BuiltinTheme),
    /// Use a custom theme with user-defined colors
    Custom(Theme),
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig::Builtin(BuiltinTheme::Warm)
    }
}

impl ThemeConfig {
    /// Resolves the config to an actual Theme
    pub fn resolve(&self) -> Theme {
        match self {
            ThemeConfig::Builtin(builtin) => builtin.theme(),
            ThemeConfig::Custom(theme) => theme.clone(),
        }
    }
}
