mod state;

use crossterm::event::KeyCode;

pub use state::{
    App, CategoryField, DeleteTickrPopup, EditTickrPopup, NewCategoryPopup, NewTickrField,
    NewTickrPopup, UpdatePopup,
};

/// Possible input events the app reacts to.
pub enum AppEvent {
    Tick,
    KeyPress(KeyCode),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusMode {
    TabBar,
    Content,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppView {
    Dashboard,
    Projects,
    Tickrs,
    ProjectTickrs,
    WorkedProjects,
    Timeline,
    Categories,
    TickrDetail,
    Help,
}

const TABS: [AppView; 6] = [
    AppView::Dashboard,
    AppView::Projects,
    AppView::Tickrs,
    AppView::WorkedProjects,
    AppView::Timeline,
    AppView::Categories,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkedRange {
    Today,
    Week,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineRange {
    Day,
    Week,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectSummary {
    pub total_seconds: i64,
    pub ended: usize,
    pub open: usize,
}
