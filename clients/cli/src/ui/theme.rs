//! Theme system for CLI customization
//!
//! Provides theme management and rotation functionality

use serde::{Deserialize, Serialize};
use ratatui::prelude::Color;

/// Theme configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub colors: ColorScheme,
    pub ui: UIStyles,
    pub sharing: SharingConfig,
}

/// Color scheme for the theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub text: String,
    pub success: String,
    pub error: String,
    pub warning: String,
    pub info: String,
}

/// UI styling options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIStyles {
    pub border_style: String,
    pub progress_bar_style: String,
    pub logo_style: String,
}

/// Sharing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConfig {
    pub public: bool,
    pub shareable: bool,
    pub tags: Vec<String>,
}

impl Theme {
    /// Convert hex color string to ratatui Color
    pub fn hex_to_color(&self, hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
        Color::White // Fallback
    }

    /// Get primary color as ratatui Color
    pub fn primary_color(&self) -> Color {
        self.hex_to_color(&self.colors.primary)
    }

    /// Get secondary color as ratatui Color
    pub fn secondary_color(&self) -> Color {
        self.hex_to_color(&self.colors.secondary)
    }

    /// Get background color as ratatui Color
    pub fn background_color(&self) -> Color {
        self.hex_to_color(&self.colors.background)
    }

    /// Get text color as ratatui Color
    pub fn text_color(&self) -> Color {
        self.hex_to_color(&self.colors.text)
    }

    /// Get success color as ratatui Color
    pub fn success_color(&self) -> Color {
        self.hex_to_color(&self.colors.success)
    }

    /// Get error color as ratatui Color
    pub fn error_color(&self) -> Color {
        self.hex_to_color(&self.colors.error)
    }

    /// Get warning color as ratatui Color
    pub fn warning_color(&self) -> Color {
        self.hex_to_color(&self.colors.warning)
    }

    /// Get info color as ratatui Color
    pub fn info_color(&self) -> Color {
        self.hex_to_color(&self.colors.info)
    }
}

/// Theme manager for handling theme rotation and management
#[derive(Debug)]
pub struct ThemeManager {
    pub current_theme_index: usize,
    pub themes: Vec<Theme>,
}

impl ThemeManager {
    /// Create a new theme manager with built-in themes
    pub fn new() -> Self {
        let themes = vec![
            Self::default_theme(),
            Self::cyberpunk_theme(),
            Self::professional_theme(),
            Self::retro_theme(),
            Self::minimal_theme(),
        ];

        Self {
            current_theme_index: 0,
            themes,
        }
    }

    /// Get the current theme
    pub fn current_theme(&self) -> &Theme {
        &self.themes[self.current_theme_index]
    }

    /// Rotate to the next theme
    pub fn next_theme(&mut self) {
        self.current_theme_index = (self.current_theme_index + 1) % self.themes.len();
    }

    /// Rotate to the previous theme
    pub fn previous_theme(&mut self) {
        if self.current_theme_index == 0 {
            self.current_theme_index = self.themes.len() - 1;
        } else {
            self.current_theme_index -= 1;
        }
    }

    /// Set theme by name
    pub fn set_theme_by_name(&mut self, name: &str) -> bool {
        if let Some(index) = self.themes.iter().position(|t| t.name == name) {
            self.current_theme_index = index;
            true
        } else {
            false
        }
    }

    /// Get theme names for display
    pub fn theme_names(&self) -> Vec<String> {
        self.themes.iter().map(|t| t.name.clone()).collect()
    }

    /// Default theme (original colors)
    fn default_theme() -> Theme {
        Theme {
            name: "Vibrant Blue".to_string(),
            author: "Nexus Team".to_string(),
            version: "1.0.0".to_string(),
            description: "Bright and vibrant blue theme".to_string(),
            colors: ColorScheme {
                primary: "#00bfff".to_string(),    // Deep sky blue
                secondary: "#ff6b6b".to_string(),  // Coral red
                background: "#1a1a1a".to_string(), // Dark gray
                text: "#ffffff".to_string(),       // White
                success: "#00ff00".to_string(),    // Bright green
                error: "#ff0000".to_string(),      // Red
                warning: "#ffff00".to_string(),     // Yellow
                info: "#0080ff".to_string(),        // Blue
            },
            ui: UIStyles {
                border_style: "double".to_string(),
                progress_bar_style: "block".to_string(),
                logo_style: "ascii_art".to_string(),
            },
            sharing: SharingConfig {
                public: true,
                shareable: true,
                tags: vec!["default".to_string(), "official".to_string()],
            },
        }
    }

    /// Cyberpunk theme
    fn cyberpunk_theme() -> Theme {
        Theme {
            name: "Cyberpunk Neon".to_string(),
            author: "Nexus Team".to_string(),
            version: "1.0.0".to_string(),
            description: "Futuristic neon colors inspired by cyberpunk aesthetics".to_string(),
            colors: ColorScheme {
                primary: "#00ff41".to_string(),     // Matrix green
                secondary: "#ff0080".to_string(),   // Hot pink
                background: "#0a0a0a".to_string(),  // Very dark
                text: "#ffffff".to_string(),        // White
                success: "#00ff00".to_string(),     // Bright green
                error: "#ff0000".to_string(),       // Red
                warning: "#ffff00".to_string(),      // Yellow
                info: "#00ffff".to_string(),         // Cyan
            },
            ui: UIStyles {
                border_style: "double".to_string(),
                progress_bar_style: "block".to_string(),
                logo_style: "ascii_art".to_string(),
            },
            sharing: SharingConfig {
                public: true,
                shareable: true,
                tags: vec!["cyberpunk".to_string(), "neon".to_string(), "matrix".to_string()],
            },
        }
    }

    /// Professional theme
    fn professional_theme() -> Theme {
        Theme {
            name: "Ocean Blue".to_string(),
            author: "Nexus Team".to_string(),
            version: "1.0.0".to_string(),
            description: "Vibrant ocean-inspired blue theme".to_string(),
            colors: ColorScheme {
                primary: "#0066cc".to_string(),     // Bright blue
                secondary: "#ff6600".to_string(),  // Orange
                background: "#001122".to_string(),  // Dark blue
                text: "#ffffff".to_string(),        // White
                success: "#00cc66".to_string(),     // Green
                error: "#ff3366".to_string(),       // Red
                warning: "#ffcc00".to_string(),     // Yellow
                info: "#00ccff".to_string(),        // Light blue
            },
            ui: UIStyles {
                border_style: "single".to_string(),
                progress_bar_style: "line".to_string(),
                logo_style: "text".to_string(),
            },
            sharing: SharingConfig {
                public: true,
                shareable: true,
                tags: vec!["professional".to_string(), "business".to_string(), "clean".to_string()],
            },
        }
    }

    /// Retro theme
    fn retro_theme() -> Theme {
        Theme {
            name: "Retro Rainbow".to_string(),
            author: "Nexus Team".to_string(),
            version: "1.0.0".to_string(),
            description: "80s computer terminal vibes with vibrant colors".to_string(),
            colors: ColorScheme {
                primary: "#ff6600".to_string(),      // Bright orange
                secondary: "#00ff00".to_string(),    // Bright green
                background: "#000000".to_string(),   // Black
                text: "#ff6600".to_string(),         // Orange
                success: "#00ff00".to_string(),      // Green
                error: "#ff0000".to_string(),        // Red
                warning: "#ffff00".to_string(),      // Yellow
                info: "#00ffff".to_string(),         // Cyan
            },
            ui: UIStyles {
                border_style: "single".to_string(),
                progress_bar_style: "block".to_string(),
                logo_style: "ascii_art".to_string(),
            },
            sharing: SharingConfig {
                public: true,
                shareable: true,
                tags: vec!["retro".to_string(), "80s".to_string(), "terminal".to_string()],
            },
        }
    }

    /// Minimal theme
    fn minimal_theme() -> Theme {
        Theme {
            name: "Sunset Gradient".to_string(),
            author: "Nexus Team".to_string(),
            version: "1.0.0".to_string(),
            description: "Warm sunset colors with vibrant gradients".to_string(),
            colors: ColorScheme {
                primary: "#ff6600".to_string(),      // Orange
                secondary: "#ff3366".to_string(),    // Pink
                background: "#1a0a0a".to_string(),   // Dark red
                text: "#ffffff".to_string(),         // White
                success: "#00ff66".to_string(),      // Green
                error: "#ff0066".to_string(),        // Red
                warning: "#ffcc00".to_string(),      // Yellow
                info: "#0066ff".to_string(),         // Blue
            },
            ui: UIStyles {
                border_style: "single".to_string(),
                progress_bar_style: "line".to_string(),
                logo_style: "text".to_string(),
            },
            sharing: SharingConfig {
                public: true,
                shareable: true,
                tags: vec!["minimal".to_string(), "clean".to_string(), "subtle".to_string()],
            },
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
