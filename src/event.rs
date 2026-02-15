use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};

use crate::app::{App, AppEvent};

/// Polls for crossterm events and maps them to `AppEvent`s.
pub fn poll(timeout: Duration) -> Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(None);
            }
            return Ok(Some(AppEvent::KeyPress(key.code)));
        }
    }
    Ok(Some(AppEvent::Tick))
}

/// Runs the main event loop.
pub fn run(app: &mut App, terminal: &mut crate::tui::Terminal) -> Result<()> {
    let tick_rate = Duration::from_millis(250);

    while app.running {
        terminal.draw(|frame| crate::ui::draw(frame, app))?;

        if let Some(event) = poll(tick_rate)? {
            app.update(event);
        }
    }
    Ok(())
}
