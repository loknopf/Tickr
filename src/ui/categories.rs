use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
};

use crate::app::App;
use super::helpers::hex_to_color;
use super::theme::Theme;

pub fn build_categories_text(app: &App) -> Text<'_> {
    if let Some(status) = &app.status {
        return Text::from(status.as_str());
    }
    if app.categories_list.is_empty() {
        return Text::from("No categories found. Press 'n' to create one.");
    }

    let mut lines = app
        .categories_list
        .iter()
        .enumerate()
        .map(|(index, category)| {
            let selected = index == app.selected_category_index;
            let marker_style = if selected {
                Style::default().fg(Theme::highlight()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::dim())
            };
            let name_style = hex_to_color(&category.color)
                .map(|color| Style::default().fg(color).add_modifier(Modifier::BOLD))
                .unwrap_or_else(|| Style::default().fg(Theme::text()));
            Line::from(vec![
                Span::styled(if selected { "> " } else { "  " }, marker_style),
                Span::styled(category.name.as_str(), name_style),
                Span::raw("  "),
                Span::styled(
                    category.color.as_str(),
                    Style::default().fg(Theme::dim()),
                ),
            ])
        })
        .collect::<Vec<_>>();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "n: New category   esc: Back",
        Style::default().fg(Theme::dim()),
    )));

    Text::from(lines)
}
