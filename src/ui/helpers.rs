use chrono::Duration;
use ratatui::style::Color;

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds().max(0);
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub fn clamp_name(value: &str, width: usize) -> String {
    let value_len = value.chars().count();
    if value_len <= width {
        return format!("{value:<width$}", width = width);
    }
    let trimmed = value
        .chars()
        .take(width.saturating_sub(2))
        .collect::<String>();
    format!("{trimmed}..")
}

pub fn hex_to_color(value: &str) -> Option<Color> {
    let hex = value.trim().strip_prefix('#').unwrap_or(value.trim());
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}
