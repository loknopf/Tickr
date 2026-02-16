use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
};

use super::helpers::format_duration;
use super::theme::Theme;
use crate::app::{App, TimelineRange};

struct DayTimeline {
    date: NaiveDate,
    hours: [u32; 24],
    total_seconds: i64,
}

pub fn build_timeline_text(app: &App) -> Text<'_> {
    let now = Local::now();
    let mut lines = Vec::new();

    let (title, days) = match app.timeline_range {
        TimelineRange::Day => ("Day", vec![now.date_naive()]),
        TimelineRange::Week => {
            let start = now.date_naive() - Duration::days(6);
            let days = (0..7)
                .map(|offset| start + Duration::days(offset))
                .collect::<Vec<_>>();
            ("Week", days)
        }
    };

    lines.push(Line::from(Span::styled(
        format!("  Timeline ({title})"),
        Style::default()
            .fg(Theme::accent())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let timelines = build_day_timelines(&days, app, now);

    match app.timeline_range {
        TimelineRange::Day => {
            let timeline = timelines.first();
            if let Some(timeline) = timeline {
                lines.push(Line::from(Span::styled(
                    format!("  Date: {}", timeline.date.format("%Y-%m-%d")),
                    Style::default().fg(Theme::secondary()),
                )));
                lines.push(Line::from(Span::styled(
                    format!(
                        "  Total: {}",
                        format_duration(Duration::seconds(timeline.total_seconds.max(0)))
                    ),
                    Style::default().fg(Theme::text()),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("  Hours: {}", hour_markers()),
                    Style::default().fg(Theme::dim()),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  Work : {}", bar_for_hours(&timeline.hours)),
                    Style::default().fg(Theme::text()),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Legend: . none  : <15m  = <30m  + <45m  # 45m+",
                    Style::default().fg(Theme::dim()),
                )));
            } else {
                lines.push(Line::from("  No data."));
            }
        }
        TimelineRange::Week => {
            lines.push(Line::from(Span::styled(
                "  Hours: |   |   |   |   |   |",
                Style::default().fg(Theme::dim()),
            )));
            lines.push(Line::from(""));
            for timeline in timelines {
                let label = timeline.date.format("%a %m-%d").to_string();
                let total = format_duration(Duration::seconds(timeline.total_seconds.max(0)));
                lines.push(Line::from(Span::styled(
                    format!("  {label}  {}  {total}", bar_for_hours(&timeline.hours)),
                    Style::default().fg(Theme::text()),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Legend: . none  : <15m  = <30m  + <45m  # 45m+",
                Style::default().fg(Theme::dim()),
            )));
        }
    }

    Text::from(lines)
}

fn build_day_timelines(days: &[NaiveDate], app: &App, now: DateTime<Local>) -> Vec<DayTimeline> {
    let mut timelines = Vec::new();

    for day in days {
        let mut timeline = DayTimeline {
            date: *day,
            hours: [0; 24],
            total_seconds: 0,
        };
        let day_start = local_start_of_day(*day);
        let day_end = day_start + Duration::days(1);

        for tickr in &app.tickrs {
            for interval in &tickr.intervals {
                let start = interval.start_time;
                let end = interval.end_time.unwrap_or(now);
                add_interval_to_day(&mut timeline, start, end, day_start, day_end);
            }
        }
        timelines.push(timeline);
    }

    timelines
}

fn add_interval_to_day(
    timeline: &mut DayTimeline,
    start: DateTime<Local>,
    end: DateTime<Local>,
    day_start: DateTime<Local>,
    day_end: DateTime<Local>,
) {
    if end <= day_start || start >= day_end {
        return;
    }

    let overlap_start = if start > day_start { start } else { day_start };
    let overlap_end = if end < day_end { end } else { day_end };
    let overlap_seconds = overlap_end.signed_duration_since(overlap_start).num_seconds();
    if overlap_seconds <= 0 {
        return;
    }
    timeline.total_seconds += overlap_seconds;

    for hour in 0..24 {
        let hour_start = day_start + Duration::hours(hour);
        let hour_end = hour_start + Duration::hours(1);
        if overlap_end > hour_start && overlap_start < hour_end {
            let segment_start = if overlap_start > hour_start {
                overlap_start
            } else {
                hour_start
            };
            let segment_end = if overlap_end < hour_end { overlap_end } else { hour_end };
            let seconds = segment_end
                .signed_duration_since(segment_start)
                .num_seconds()
                .max(0) as u32;
            timeline.hours[hour as usize] = timeline.hours[hour as usize].saturating_add(seconds);
        }
    }
}

fn bar_for_hours(hours: &[u32; 24]) -> String {
    hours.iter().map(|&secs| hour_fill(secs)).collect()
}

fn hour_fill(seconds: u32) -> char {
    match seconds {
        0 => '.',
        1..=899 => ':',
        900..=1799 => '=',
        1800..=2699 => '+',
        _ => '#',
    }
}

fn hour_markers() -> String {
    let mut marker = String::new();
    for hour in 0..24 {
        if hour % 4 == 0 {
            marker.push('|');
        } else {
            marker.push(' ');
        }
    }
    marker
}

fn local_start_of_day(date: NaiveDate) -> DateTime<Local> {
    let naive = date.and_hms_opt(0, 0, 0).expect("valid time");
    match Local.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(dt, _) => dt,
        chrono::LocalResult::None => Local.from_utc_datetime(&naive),
    }
}
