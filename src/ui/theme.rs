use ratatui::style::Color;

/// Unified color theme for the application
pub struct Theme;

impl Theme {
    /// Primary branding color
    pub fn primary() -> Color {
        Color::Magenta
    }

    /// Secondary/border color
    pub fn secondary() -> Color {
        Color::Cyan
    }

    /// Success/completed status
    pub fn success() -> Color {
        Color::Green
    }

    /// Running/active status
    pub fn active() -> Color {
        Color::LightGreen
    }

    /// Warning/pending status
    pub fn warn() -> Color {
        Color::Yellow
    }

    /// Error/ended status
    pub fn ended() -> Color {
        Color::Blue
    }

    /// Selection/highlight
    pub fn highlight() -> Color {
        Color::Cyan
    }

    /// Selection marker/arrow
    pub fn selection_marker() -> Color {
        Color::Green
    }

    /// Dimmed/inactive text
    pub fn dim() -> Color {
        Color::DarkGray
    }

    /// Normal text
    pub fn text() -> Color {
        Color::White
    }

    /// Accent for numbers/counts
    pub fn accent() -> Color {
        Color::LightBlue
    }
}
