use std::u32;

use chrono::{DateTime, Local};

pub type TickrId = u32;
pub type ProjectId = u32;
pub type CategoryId = u32;
pub type IntervalId = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Project {
    pub id: Option<ProjectId>,
    pub name: String,
    pub created_at: DateTime<Local>
}

pub(crate) enum ProjectQuery {
    All,
    ByName(String),
}

///A single Tickr is a single entry belonging to a project
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tickr {
    pub id: Option<TickrId>,
    pub project_id: ProjectId,
    pub description: String,
    pub category_id: Option<CategoryId>,
    pub intervals: Vec<Interval>,
}

pub(crate) enum TickrQuery {
    All,
    ByProject(String),
    ByProjectId(ProjectId),
    ByTimeRange(DateTime<Local>, DateTime<Local>),
}

pub(crate) struct TickrCategory {
    pub name: String,
    pub id: CategoryId,
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Interval {
    pub id: Option<IntervalId>,
    pub entry_id: TickrId,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
}