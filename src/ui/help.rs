use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
};

use super::theme::Theme;
use crate::app::App;

pub fn build_help_text(_app: &App) -> Text<'_> {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "Key bindings",
        Style::default()
            .fg(Theme::accent())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    lines.push(section_title("Global"));
    lines.extend(section_lines(&[
        "q: Quit",
        "?: Toggle help",
        "Tab: Toggle focus (tab bar / content)",
        "Left/Right: Navigate tabs (tab bar focus)",
        "Enter: Activate tab (tab bar focus)",
        "h/p/t/w/l/c: Quick nav",
        "r: Refresh current view",
        "esc: Back",
    ]));

    lines.push(Line::from(""));
    lines.push(section_title("Lists"));
    lines.extend(section_lines(&[
        "Up/Down: Move selection",
        "Enter: Open",
    ]));

    lines.push(Line::from(""));
    lines.push(section_title("Projects"));
    lines.extend(section_lines(&["/: Search projects"]));

    lines.push(Line::from(""));
    lines.push(section_title("Tickrs"));
    lines.extend(section_lines(&[
        "space: Start/End task",
        "s: Stop running task",
        "g: Go to project (detail)",
        "e: Edit task (detail)",
    ]));

    lines.push(Line::from(""));
    lines.push(section_title("Create"));
    lines.extend(section_lines(&[
        "n: New task (projects/tickrs) or new category (categories)",
    ]));

    lines.push(Line::from(""));
    lines.push(section_title("Worked/Timeline"));
    lines.extend(section_lines(&["Shift+Tab: Toggle day/week range"]));

    lines.push(Line::from(""));
    lines.push(section_title("Popups"));
    lines.extend(section_lines(&[
        "Edit task: Up/Down change category, Enter save, Esc cancel",
        "New category: Tab switch field, Enter save, Esc cancel",
        "New task: Tab switch field, Up/Down select, Space toggle start, Enter save, Esc cancel",
    ]));

    Text::from(lines)
}

fn section_title(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {title}"),
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    ))
}

fn section_lines(items: &[&str]) -> Vec<Line<'static>> {
    items
        .iter()
        .map(|item| {
            Line::from(Span::styled(
                format!("  - {item}"),
                Style::default().fg(Theme::text()),
            ))
        })
        .collect()
}
