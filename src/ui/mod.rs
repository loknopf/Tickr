mod dashboard;
mod detail;
mod helpers;
mod categories;
mod projects;
mod tickrs;
mod theme;

use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, Clear, Paragraph},
    Frame,
};

use crate::app::{App, AppView};
use theme::Theme;

use helpers::{format_duration, hex_to_color};

/// Renders the entire UI for a single frame.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let (title, body_text) = match app.view {
        AppView::Dashboard => (" Dashboard ", dashboard::build_dashboard_text(app)),
        AppView::Projects => (" Projects ", projects::build_projects_text(app)),
        AppView::Tickrs => (" Tickrs ", tickrs::build_tickrs_text(app, true)),
        AppView::ProjectTickrs => (
            projects::build_project_tickr_title(app),
            tickrs::build_tickrs_text(app, true),
        ),
        AppView::WorkedProjects => (" Worked ", projects::build_worked_projects_text(app)),
        AppView::Categories => (" Categories ", categories::build_categories_text(app)),
        AppView::TickrDetail => (" Task ", detail::build_tickr_detail_text(app)),
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    let header_lines = vec![Line::from(vec![
        Span::styled("  Tickr  ", Style::default().fg(Color::Black).bg(Theme::primary())),
        Span::raw(" "),
        Span::styled(
            "time tracker",
            Style::default().fg(Theme::secondary()).add_modifier(Modifier::BOLD),
        ),
    ])];
    let header = Paragraph::new(Text::from(header_lines))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Theme::secondary())),
        );
    frame.render_widget(header, layout[0]);

    let mut body_lines = vec![
        tabs_line(app),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {title}"),
            Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    body_lines.extend(body_text.lines);
    body_lines.push(Line::from(""));
    body_lines.push(Line::from(Span::styled(
        "----------------------------------------",
        Style::default().fg(Theme::dim()),
    )));
    body_lines.extend(keybinds_lines(app));
    let body = Paragraph::new(Text::from(body_lines))
        .style(Style::default().fg(Theme::text()))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Theme::secondary())),
        );
    frame.render_widget(body, layout[1]);

    let footer = Paragraph::new(Text::from(running_task_line(app)))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Theme::secondary())),
        );
    frame.render_widget(footer, layout[2]);

    if let Some(popup) = &app.edit_popup {
        render_edit_popup(frame, popup);
    }
    if let Some(popup) = &app.new_category_popup {
        render_new_category_popup(frame, popup);
    }
    if let Some(popup) = &app.new_tickr_popup {
        render_new_tickr_popup(frame, popup);
    }
}

fn render_edit_popup(frame: &mut Frame, popup: &crate::app::EditTickrPopup) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Edit task",
        Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Label: ", Style::default().fg(Theme::dim())),
        Span::styled(
            popup.label.as_str(),
            Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Category",
        Style::default().fg(Theme::dim()),
    )));

    for (index, option) in popup.categories.iter().enumerate() {
        let selected = index == popup.category_index;
        let marker_style = if selected {
            Style::default().fg(Theme::selection_marker()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::dim())
        };
        let mut name_style = if let Some(color) = option.color.as_deref().and_then(hex_to_color) {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::text())
        };
        if selected {
            name_style = name_style.add_modifier(Modifier::BOLD);
        }
        lines.push(Line::from(vec![
            Span::styled(if selected { "> " } else { "  " }, marker_style),
            Span::styled(option.name.as_str(), name_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Type to edit label. Up/Down: category. Enter: save. Esc: cancel.",
        Style::default().fg(Theme::dim()),
    )));

    let popup = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Theme::secondary()))
                .title(" Edit "),
        );
    frame.render_widget(popup, area);
}

fn render_new_category_popup(frame: &mut Frame, popup: &crate::app::NewCategoryPopup) {
    let area = centered_rect(60, 45, frame.area());
    frame.render_widget(Clear, area);

    let name_style = if popup.field == crate::app::CategoryField::Name {
        Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::text())
    };
    let color_style = if popup.field == crate::app::CategoryField::Color {
        Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::text())
    };

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "New category",
        Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Name: ", Style::default().fg(Theme::dim())),
        Span::styled(popup.name.as_str(), name_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Color: ", Style::default().fg(Theme::dim())),
        Span::styled(
            if popup.color.is_empty() {
                "#RRGGBB"
            } else {
                popup.color.as_str()
            },
            color_style,
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Type to edit. Tab: switch field. Enter: save. Esc: cancel.",
        Style::default().fg(Theme::dim()),
    )));

    let popup_widget = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Theme::secondary()))
                .title(" New Category "),
        );
    frame.render_widget(popup_widget, area);
}

fn render_new_tickr_popup(frame: &mut Frame, popup: &crate::app::NewTickrPopup) {
    let area = centered_rect(70, 75, frame.area());
    frame.render_widget(Clear, area);

    let label_active = popup.field == crate::app::NewTickrField::Label;
    let project_active = popup.field == crate::app::NewTickrField::Project;
    let category_active = popup.field == crate::app::NewTickrField::Category;
    let start_active = popup.field == crate::app::NewTickrField::StartNow;

    let arrow_style = Style::default()
        .fg(Theme::selection_marker())
        .add_modifier(Modifier::BOLD);
    let label_style = if label_active {
        Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::text())
    };
    let label_title_style = if label_active {
        Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::dim())
    };
    let project_title_style = if project_active {
        Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::dim())
    };
    let category_title_style = if category_active {
        Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::dim())
    };
    let start_style = if start_active {
        Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::text())
    };
    let start_title_style = if start_active {
        Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::dim())
    };

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "New task",
        Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(if label_active { "> " } else { "  " }, arrow_style),
        Span::styled("Label: ", label_title_style),
        Span::styled(popup.label.as_str(), label_style),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(if project_active { "> " } else { "  " }, arrow_style),
        Span::styled("Project", project_title_style),
    ]));
    for (index, option) in popup.projects.iter().enumerate() {
        let selected = index == popup.project_index;
        let marker_style = if selected {
            arrow_style
        } else {
            Style::default().fg(Theme::dim())
        };
        let name_style = if selected || project_active {
            Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::text())
        };
        lines.push(Line::from(vec![
            Span::styled(if selected { "> " } else { "  " }, marker_style),
            Span::styled(option.name.as_str(), name_style),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(if category_active { "> " } else { "  " }, arrow_style),
        Span::styled("Category", category_title_style),
    ]));
    for (index, option) in popup.categories.iter().enumerate() {
        let selected = index == popup.category_index;
        let marker_style = if selected {
            arrow_style
        } else {
            Style::default().fg(Theme::dim())
        };
        let mut name_style = if let Some(color) = option.color.as_deref().and_then(hex_to_color) {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::text())
        };
        if selected || category_active {
            name_style = name_style.add_modifier(Modifier::BOLD);
        }
        lines.push(Line::from(vec![
            Span::styled(if selected { "> " } else { "  " }, marker_style),
            Span::styled(option.name.as_str(), name_style),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(if start_active { "> " } else { "  " }, arrow_style),
        Span::styled("Start now: ", start_title_style),
        Span::styled(
            if popup.start_now { "yes" } else { "no" },
            start_style,
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Type to edit label. Tab: switch field. Up/Down: select. Space: toggle start. Enter: save. Esc: cancel.",
        Style::default().fg(Theme::dim()),
    )));

    let popup_widget = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Theme::secondary()))
                .title(" New Task "),
        );
    frame.render_widget(popup_widget, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn tabs_line(app: &App) -> Line<'_> {
    let tabs = [
        ("Home", AppView::Dashboard),
        ("Projects", AppView::Projects),
        ("Tickrs", AppView::Tickrs),
        ("Worked", AppView::WorkedProjects),
        ("Categories", AppView::Categories),
    ];

    let mut spans = Vec::new();
    for (index, (name, view)) in tabs.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        let active = match app.view {
            AppView::ProjectTickrs => *view == AppView::Tickrs,
            AppView::TickrDetail => *view == AppView::Tickrs,
            AppView::WorkedProjects => *view == AppView::WorkedProjects,
            _ => *view == app.view,
        };
        let focused = app.focus_mode == crate::app::FocusMode::TabBar && app.selected_tab_index == index;
        let style = if active {
            Style::default()
                .fg(Color::Black)
                .bg(Theme::highlight())
                .add_modifier(Modifier::BOLD)
        } else if focused {
            Style::default()
                .fg(Theme::highlight())
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Theme::dim())
        };
        spans.push(Span::styled(format!(" {name} "), style));
    }

    Line::from(spans)
}

fn running_task_line(app: &App) -> Line<'_> {
    let now = Local::now();
    let mut running: Option<(&crate::types::Tickr, &crate::types::Interval)> = None;
    for tickr in &app.tickrs {
        if let Some(interval) = tickr.intervals.iter().find(|i| i.end_time.is_none()) {
            running = Some((tickr, interval));
            break;
        }
    }

    let text = if let Some((tickr, interval)) = running {
        let project_name = app
            .projects
            .iter()
            .find(|project| project.id == Some(tickr.project_id))
            .map(|project| project.name.as_str())
            .unwrap_or("Unknown project");
        let duration = format_duration(now.signed_duration_since(interval.start_time));
        format!("{project_name} > {} > Running {duration}", tickr.description)
    } else {
        "No task running".to_string()
    };

    Line::from(Span::styled(text, Style::default().fg(Theme::active()).add_modifier(Modifier::BOLD)))
}

fn keybinds_lines(app: &App) -> Vec<Line<'static>> {
    let focus_hint = if app.focus_mode == crate::app::FocusMode::TabBar {
        "Tab: Switch to content  ←/→: Navigate tabs  Enter: Select"
    } else {
        "Tab: Switch to tab bar  h/p/t/w/c: Quick nav"
    };
    
    let (primary, secondary) = match app.view {
        AppView::Dashboard => (
            "h: Home  p: Projects  t: Tasks  w: Worked  c: Categories",
            "r: Refresh  q: Quit",
        ),
        AppView::Projects => (
            "Up/Down: Select  Enter: Open  n: New task",
            "r: Refresh  q: Quit",
        ),
        AppView::Tickrs => (
            "Up/Down: Select  Enter: Detail  space: Start/End",
            "r: Refresh  q: Quit",
        ),
        AppView::ProjectTickrs => (
            "Up/Down: Select  Enter: Detail  space: Start/End  n: New task",
            "esc: Back  r: Refresh  q: Quit",
        ),
        AppView::WorkedProjects => (
            "Up/Down: Select  Enter: Open  Shift+Tab: Adjust Range",
            "r: Refresh  q: Quit",
        ),
        AppView::Categories => (
            "Up/Down: Select  n: New",
            "esc: Back  r: Refresh  q: Quit",
        ),
        AppView::TickrDetail => (
            "space: Start/End  s: Stop  g: Project  e: Edit",
            "esc: Back  q: Quit",
        ),
    };
    vec![
        Line::from(Span::styled(
            focus_hint,
            Style::default().fg(Theme::highlight()),
        )),
        Line::from(Span::styled(
            primary,
            Style::default().fg(Theme::dim()),
        )),
        Line::from(Span::styled(
            secondary,
            Style::default().fg(Theme::dim()),
        )),
    ]
}
