use chrono::{Duration, Local};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

use super::theme::Theme;
use crate::app::App;

use super::helpers::{format_duration, hex_to_color};

pub fn build_tickr_detail_text(app: &App) -> Text<'_> {
    if let Some(status) = &app.status {
        return Text::from(status.as_str());
    }
    let Some(tickr) = &app.selected_tickr else {
        return Text::from("No task selected.");
    };

    const LABEL_WIDTH: usize = 11;
    let label_style = Style::default().fg(Theme::dim());
    let label = |name: &str| {
        let label_text = format!("{name}:");
        Span::styled(
            format!("{label_text:width$}", width = LABEL_WIDTH),
            label_style,
        )
    };
    let value = |text: &str| Span::raw(text.to_string());

    let project = app
        .selected_tickr_project_name
        .as_deref()
        .unwrap_or("Unknown project");
    let category_line = if let Some(category) = app.category_for_tickr(tickr) {
        let cat_color = hex_to_color(&category.color).unwrap_or(Color::Magenta);
        Line::from(vec![
            label("Category"),
            Span::styled(
                category.name.as_str(),
                Style::default().fg(cat_color).add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![label("Category"), value("none")])
    };

    let first_start = tickr
        .intervals
        .first()
        .map(|i| i.start_time.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "pending".to_string());
    let last_end = tickr
        .intervals
        .last()
        .and_then(|i| i.end_time)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string());

    let status = if tickr.intervals.is_empty() {
        "Not started"
    } else if tickr.intervals.last().unwrap().end_time.is_none() {
        "Running"
    } else {
        "Ended"
    };
    let status_color = match status {
        "Running" => Theme::active(),
        "Ended" => Theme::ended(),
        _ => Theme::warn(),
    };

    let now = Local::now();
    let total_duration = tickr
        .intervals
        .iter()
        .fold(Duration::seconds(0), |acc, interval| {
            let end_time = interval.end_time.unwrap_or(now);
            acc + end_time.signed_duration_since(interval.start_time)
        });
    let elapsed = if tickr.intervals.is_empty() {
        "--:--:--".to_string()
    } else {
        format_duration(total_duration)
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                "Task",
                Style::default()
                    .fg(Theme::primary())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                tickr.description.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from("----------------------------------------"),
        Line::from(vec![label("Project"), value(project)]),
        category_line,
        Line::from(vec![
            label("Status"),
            Span::styled(
                status,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![label("First"), value(&first_start)]),
        Line::from(vec![
            label("Last"),
            value(&last_end.clone().unwrap_or_else(|| "open".to_string())),
        ]),
        Line::from(vec![label("Elapsed"), value(&elapsed)]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Intervals ({})", tickr.intervals.len()),
            Style::default()
                .fg(Theme::accent())
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    if tickr.intervals.is_empty() {
        lines.push(Line::from(vec![Span::styled("  none", label_style)]));
    } else {
        for (index, interval) in tickr.intervals.iter().enumerate() {
            let start = interval.start_time.format("%Y-%m-%d %H:%M").to_string();
            let (end, duration) = if let Some(end_time) = interval.end_time {
                let end = end_time.format("%Y-%m-%d %H:%M").to_string();
                let duration = format_duration(end_time.signed_duration_since(interval.start_time));
                (end, duration)
            } else {
                let end = "open".to_string();
                let duration = format_duration(now.signed_duration_since(interval.start_time));
                (end, duration)
            };
            if tickr.intervals.len() > 5 {
                if index < 2 || index >= tickr.intervals.len() - 2 {
                    lines.push(Line::from(vec![
                        Span::raw(format!("  {:>2}) {start} -> {end} ", index + 1)),
                        Span::styled(format!("({duration})"), Style::default().fg(Theme::dim())),
                    ]));
                } else if index == 2 {
                    lines.push(Line::from(vec![Span::raw("     ...")]));
                }
            } else {
                lines.push(Line::from(vec![
                    Span::raw(format!("  {:>2}) {start} -> {end} ", index + 1)),
                    Span::styled(format!("({duration})"), Style::default().fg(Theme::dim())),
                ]));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(
        "space: Start/End   s: Stop running   g: Project   e: Edit   d: Delete   esc: Back",
    ));
    Text::from(lines)
}
