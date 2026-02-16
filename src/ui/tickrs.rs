use chrono::{Duration, Local};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

use super::helpers::{format_duration, hex_to_color};
use super::theme::Theme;
use crate::app::App;

pub fn build_tickrs_text(app: &App, show_selection: bool) -> Text<'_> {
    let mut lines = Vec::new();
    
    // Show search bar if active
    if app.search_active {
        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)),
            Span::styled(&app.search_filter, Style::default().fg(Theme::text())),
            Span::styled("_", Style::default().fg(Theme::highlight())),
        ]));
        lines.push(Line::from(Span::styled(
            "Type to search, Enter to apply, Esc to cancel",
            Style::default().fg(Theme::dim()),
        )));
        lines.push(Line::from(""));
    } else if !app.search_filter.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Filtered: ", Style::default().fg(Theme::accent())),
            Span::styled(&app.search_filter, Style::default().fg(Theme::text())),
            Span::styled(" (press / to edit, Esc to clear)", Style::default().fg(Theme::dim())),
        ]));
        lines.push(Line::from(""));
    }
    
    if let Some(status) = &app.status {
        lines.push(Line::from(status.as_str()));
        return Text::from(lines);
    }
    
    let tickrs_to_display = app.filtered_tickrs();
    
    if tickrs_to_display.is_empty() {
        if app.search_filter.is_empty() {
            lines.push(Line::from("No tickrs found. Press 'r' to refresh."));
        } else {
            lines.push(Line::from(vec![
                Span::styled("No tasks match '", Style::default().fg(Theme::dim())),
                Span::styled(&app.search_filter, Style::default().fg(Theme::text())),
                Span::styled("'. Press Esc to clear filter.", Style::default().fg(Theme::dim())),
            ]));
        }
        return Text::from(lines);
    }
    
    let tickr_lines = tickrs_to_display
        .iter()
        .enumerate()
        .map(|(index, tickr)| {
            let intervals = &tickr.intervals;
            let interval_text = if intervals.is_empty() {
                "0 intervals, --:--:--".to_string()
            } else {
                let now = Local::now();
                let total_duration =
                    intervals
                        .iter()
                        .fold(Duration::seconds(0), |acc, interval| {
                            let end_time = interval.end_time.unwrap_or(now);
                            acc + end_time.signed_duration_since(interval.start_time)
                        });
                let elapsed = format_duration(total_duration);
                let count = intervals.len();
                let label = if count == 1 { "interval" } else { "intervals" };
                format!("{count} {label}, {elapsed}")
            };
            let selected = show_selection && index == app.selected_tickr_index;
            let line_style = if selected {
                Style::default()
                    .fg(Theme::highlight())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let marker_style = if selected {
                Style::default().fg(Theme::selection_marker())
            } else {
                Style::default().fg(Theme::dim())
            };
            let mut spans = vec![
                Span::styled(if selected { "> " } else { "  " }, marker_style),
                Span::styled(format!("[{interval_text}] "), line_style),
            ];
            if let Some(category) = app.category_for_tickr(tickr) {
                let cat_color = hex_to_color(&category.color).unwrap_or(Color::Magenta);
                spans.push(Span::styled(
                    format!("[{}] ", category.name),
                    Style::default().fg(cat_color).add_modifier(Modifier::BOLD),
                ));
            }
            spans.push(Span::styled(&tickr.description, line_style));
            Line::from(spans)
        })
        .collect::<Vec<_>>();
    
    lines.extend(tickr_lines);
    Text::from(lines)
}
