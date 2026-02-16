use chrono::{Duration, Local};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

use super::helpers::{clamp_name, format_duration, hex_to_color};
use super::theme::Theme;
use crate::app::App;

pub fn build_dashboard_text(app: &App) -> Text<'_> {
    let mut lines = Vec::new();

    // Welcome section
    let now = Local::now();
    lines.push(Line::from(Span::styled(
        format!("  Welcome to Tickr - {}", now.format("%A, %B %e, %Y")),
        Style::default()
            .fg(Theme::accent())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Current running task section
    lines.push(Line::from(Span::styled(
        "  Current Task",
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  ─────────────",
        Style::default().fg(Theme::dim()),
    )));

    let mut running_found = false;
    for tickr in &app.tickrs {
        if let Some(interval) = tickr.intervals.iter().find(|i| i.end_time.is_none()) {
            let project_name = app
                .projects
                .iter()
                .find(|project| project.id == Some(tickr.project_id))
                .map(|project| project.name.as_str())
                .unwrap_or("Unknown");
            let duration = format_duration(now.signed_duration_since(interval.start_time));

            lines.push(Line::from(vec![
                Span::styled(
                    "  ● ",
                    Style::default()
                        .fg(Theme::active())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &tickr.description,
                    Style::default()
                        .fg(Theme::text())
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("Project: ", Style::default().fg(Theme::dim())),
                Span::styled(project_name, Style::default().fg(Theme::highlight())),
                Span::raw("  "),
                Span::styled("Time: ", Style::default().fg(Theme::dim())),
                Span::styled(duration, Style::default().fg(Theme::active())),
            ]));
            running_found = true;
            break;
        }
    }

    if !running_found {
        lines.push(Line::from(Span::styled(
            "  No task currently running",
            Style::default().fg(Theme::dim()),
        )));
    }
    lines.push(Line::from(""));

    // Today's summary section
    lines.push(Line::from(Span::styled(
        "  Today's Summary",
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  ────────────────",
        Style::default().fg(Theme::dim()),
    )));

    let today_start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap();
    let today_tickrs: Vec<_> = app
        .tickrs
        .iter()
        .filter(|tickr| {
            tickr
                .intervals
                .iter()
                .any(|interval| interval.start_time >= today_start)
        })
        .collect();

    let today_duration = today_tickrs
        .iter()
        .fold(Duration::seconds(0), |acc, tickr| {
            let tickr_duration =
                tickr
                    .intervals
                    .iter()
                    .fold(Duration::seconds(0), |acc2, interval| {
                        if interval.start_time >= today_start {
                            let end_time = interval.end_time.unwrap_or(now);
                            acc2 + end_time.signed_duration_since(interval.start_time)
                        } else {
                            acc2
                        }
                    });
            acc + tickr_duration
        });

    let today_projects: std::collections::HashSet<_> =
        today_tickrs.iter().map(|tickr| tickr.project_id).collect();

    lines.push(Line::from(vec![
        Span::styled("  Total time: ", Style::default().fg(Theme::dim())),
        Span::styled(
            format_duration(today_duration),
            Style::default()
                .fg(Theme::accent())
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Tasks worked: ", Style::default().fg(Theme::dim())),
        Span::styled(
            format!("{}", today_tickrs.len()),
            Style::default()
                .fg(Theme::success())
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Projects: ", Style::default().fg(Theme::dim())),
        Span::styled(
            format!("{}", today_projects.len()),
            Style::default()
                .fg(Theme::highlight())
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    // Quick Stats section
    lines.push(Line::from(Span::styled(
        "  Quick Stats",
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  ────────────",
        Style::default().fg(Theme::dim()),
    )));

    lines.push(Line::from(vec![
        Span::styled("  Total Projects: ", Style::default().fg(Theme::dim())),
        Span::styled(
            format!("{}", app.projects.len()),
            Style::default()
                .fg(Theme::text())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Total Tasks: ", Style::default().fg(Theme::dim())),
        Span::styled(
            format!("{}", app.tickrs.len()),
            Style::default()
                .fg(Theme::text())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Categories: ", Style::default().fg(Theme::dim())),
        Span::styled(
            format!("{}", app.categories.len()),
            Style::default()
                .fg(Theme::text())
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    // Recent Projects section
    lines.push(Line::from(Span::styled(
        "  Recent Projects",
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  ────────────────",
        Style::default().fg(Theme::dim()),
    )));

    if app.projects.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No projects yet",
            Style::default().fg(Theme::dim()),
        )));
    } else {
        let recent_projects = app.projects.iter().take(5);
        for project in recent_projects {
            let summary = app.project_summary_for(project);
            let name = clamp_name(&project.name, 30);
            let total = format_duration(Duration::seconds(summary.total_seconds.max(0)));

            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(Theme::dim())),
                Span::styled(name, Style::default().fg(Theme::text())),
                Span::raw(" "),
                Span::styled(format!("[{}]", total), Style::default().fg(Theme::accent())),
            ]));
        }
    }
    lines.push(Line::from(""));

    // Recent Tasks section
    lines.push(Line::from(Span::styled(
        "  Recent Tasks",
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  ─────────────",
        Style::default().fg(Theme::dim()),
    )));

    if app.tickrs.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No tasks yet",
            Style::default().fg(Theme::dim()),
        )));
    } else {
        let recent_tickrs = app.tickrs.iter().take(5);
        for tickr in recent_tickrs {
            let total_duration =
                tickr
                    .intervals
                    .iter()
                    .fold(Duration::seconds(0), |acc, interval| {
                        let end_time = interval.end_time.unwrap_or(now);
                        acc + end_time.signed_duration_since(interval.start_time)
                    });

            let mut spans = vec![Span::styled("  • ", Style::default().fg(Theme::dim()))];

            if let Some(category) = app.category_for_tickr(tickr) {
                let cat_color = hex_to_color(&category.color).unwrap_or(Color::Magenta);
                spans.push(Span::styled(
                    format!("[{}] ", category.name),
                    Style::default().fg(cat_color).add_modifier(Modifier::BOLD),
                ));
            }

            let description = clamp_name(&tickr.description, 35);
            spans.push(Span::styled(
                description,
                Style::default().fg(Theme::text()),
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[{}]", format_duration(total_duration)),
                Style::default().fg(Theme::accent()),
            ));

            lines.push(Line::from(spans));
        }
    }

    Text::from(lines)
}
