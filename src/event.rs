use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};

use crate::app::{App, AppEvent};

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// Polls for crossterm events and maps them to `AppEvent`s.
    pub fn poll(&mut self, timeout: Duration) -> Result<Option<AppEvent>> {
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
    pub fn run(&mut self, app: &mut App, terminal: &mut crate::tui::Terminal) -> Result<()> {
        let tick_rate = Duration::from_millis(250);

        while app.running {
            terminal.draw(|frame| crate::ui::draw(frame, app))?;

            if let Some(event) = self.poll(tick_rate)? {
                app.update(event);
            }
        }
        Ok(())
    }
}
