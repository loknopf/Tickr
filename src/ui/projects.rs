use chrono::Duration;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
};

use super::helpers::{clamp_name, format_duration};
use super::theme::Theme;
use crate::app::{App, WorkedRange};

pub fn build_projects_text(app: &App) -> Text<'_> {
    if let Some(status) = &app.status {
        return Text::from(status.as_str());
    }
    if app.projects.is_empty() {
        return Text::from("No projects found. Press 'r' to refresh.");
    }
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "  {:<24} {:>8} {:>5} {:>5}",
            "Project", "Total", "End", "Open"
        ),
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "  {:<24} {:>8} {:>5} {:>5}",
            "------------------------", "--------", "-----", "-----"
        ),
        Style::default().fg(Theme::dim()),
    )));
    let project_lines = app
        .projects
        .iter()
        .enumerate()
        .map(|(index, project)| {
            let summary = app.project_summary_for(project);
            let name = clamp_name(project.name.as_str(), 24);
            let total = format_duration(Duration::seconds(summary.total_seconds.max(0)));
            let total_text = format!("{:>8}", total);
            let ended_text = format!("{:>5}", summary.ended);
            let open_text = format!("{:>5}", summary.open);
            let selected = index == app.selected_project_index;
            let name_style = if selected {
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
            Line::from(vec![
                Span::styled(if selected { "> " } else { "  " }, marker_style),
                Span::styled(name, name_style),
                Span::raw(" "),
                Span::styled(total_text, Style::default().fg(Theme::accent())),
                Span::raw(" "),
                Span::styled(ended_text, Style::default().fg(Theme::success())),
                Span::raw(" "),
                Span::styled(open_text, Style::default().fg(Theme::warn())),
            ])
        })
        .collect::<Vec<_>>();
    lines.extend(project_lines);
    Text::from(lines)
}

pub fn build_project_tickr_title(app: &App) -> &str {
    let Some(project) = &app.selected_project else {
        return " Project Tickrs ";
    };
    &project.name
}

pub fn build_worked_projects_text(app: &App) -> Text<'_> {
    if let Some(status) = &app.status {
        return Text::from(status.as_str());
    }
    if app.worked_projects.is_empty() {
        let label = worked_range_label(app.worked_range);
        return Text::from(format!("No projects worked on {label}."));
    }

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("  Worked on: {}", worked_range_label(app.worked_range)),
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {:<28}", "Project"),
        Style::default()
            .fg(Theme::secondary())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {:<28}", "----------------------------"),
        Style::default().fg(Theme::dim()),
    )));

    let project_lines = app
        .worked_projects
        .iter()
        .enumerate()
        .map(|(index, project)| {
            let name = clamp_name(project.name.as_str(), 28);
            let selected = index == app.selected_worked_project_index;
            let name_style = if selected {
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
            Line::from(vec![
                Span::styled(if selected { "> " } else { "  " }, marker_style),
                Span::styled(name, name_style),
            ])
        })
        .collect::<Vec<_>>();
    lines.extend(project_lines);
    Text::from(lines)
}

fn worked_range_label(range: WorkedRange) -> &'static str {
    match range {
        WorkedRange::Today => "today",
        WorkedRange::Week => "this week",
    }
}
