use std::collections::{HashMap, HashSet};

use crossterm::event::KeyCode;
use rusqlite::Connection;

use crate::db;
use crate::types::{CategoryId, Project, ProjectId, Tickr, TickrCategory, TickrId};

use super::{AppEvent, AppView, FocusMode, ProjectSummary, TABS, TimelineRange, WorkedRange};

/// The top-level application state.
pub struct App {
    pub running: bool,
    pub running_tickr: Option<TickrId>,
    pub db: Connection,
    pub view: AppView,
    view_history: Vec<AppView>,
    pub projects: Vec<Project>,
    pub worked_projects: Vec<Project>,
    pub tickrs: Vec<Tickr>,
    pub categories_list: Vec<TickrCategory>,
    pub status: Option<String>,
    pub selected_project_index: usize,
    pub selected_project: Option<Project>,
    pub selected_worked_project_index: usize,
    pub selected_tickr_index: usize,
    pub selected_tickr: Option<Tickr>,
    pub selected_tickr_project_name: Option<String>,
    pub selected_category_index: usize,
    pub tickr_detail_parent: AppView,
    pub project_summaries: HashMap<ProjectId, ProjectSummary>,
    pub categories: HashMap<CategoryId, TickrCategory>,
    pub worked_range: WorkedRange,
    pub timeline_range: TimelineRange,
    pub focus_mode: FocusMode,
    pub selected_tab_index: usize,
    pub projects_search_query: String,
    pub projects_search_active: bool,
    pub edit_popup: Option<EditTickrPopup>,
    pub new_category_popup: Option<NewCategoryPopup>,
    pub new_tickr_popup: Option<NewTickrPopup>,
}

#[derive(Clone, Debug)]
pub struct CategoryOption {
    pub id: Option<CategoryId>,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ProjectOption {
    pub id: ProjectId,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct EditTickrPopup {
    pub tickr_id: TickrId,
    pub label: String,
    pub category_index: usize,
    pub categories: Vec<CategoryOption>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CategoryField {
    Name,
    Color,
}

#[derive(Clone, Debug)]
pub struct NewCategoryPopup {
    pub name: String,
    pub color: String,
    pub field: CategoryField,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewTickrField {
    Label,
    Project,
    Category,
    StartNow,
}

#[derive(Clone, Debug)]
pub struct NewTickrPopup {
    pub label: String,
    pub project_index: usize,
    pub category_index: usize,
    pub projects: Vec<ProjectOption>,
    pub categories: Vec<CategoryOption>,
    pub start_now: bool,
    pub field: NewTickrField,
}

impl EditTickrPopup {
    fn select_prev(&mut self) {
        if self.categories.is_empty() {
            return;
        }
        if self.category_index == 0 {
            self.category_index = self.categories.len() - 1;
        } else {
            self.category_index -= 1;
        }
    }

    fn select_next(&mut self) {
        if self.categories.is_empty() {
            return;
        }
        self.category_index = (self.category_index + 1) % self.categories.len();
    }
}

impl App {
    pub fn new(db: Connection) -> Self {
        let tickrs = match db::query_tickr(crate::types::TickrQuery::All, &db) {
            Ok(tickrs) => tickrs,
            Err(_) => Vec::new(),
        };
        let projects = match db::query_projects(&db) {
            Ok(projects) => projects,
            Err(_) => Vec::new(),
        };
        let running_tickr = match tickrs
            .iter()
            .find(|tickr| {
                tickr
                    .intervals
                    .last()
                    .map(|interval| interval.end_time.is_none())
                    .unwrap_or(false)
            })
            .and_then(|tickr| tickr.id)
        {
            Some(id) => Some(id),
            None => None,
        };
        let mut app = Self {
            running: true,
            running_tickr,
            db,
            view: AppView::Dashboard,
            view_history: Vec::new(),
            projects,
            worked_projects: Vec::new(),
            tickrs,
            categories_list: Vec::new(),
            status: None,
            selected_project_index: 0,
            selected_project: None,
            selected_worked_project_index: 0,
            selected_tickr_index: 0,
            selected_tickr: None,
            selected_tickr_project_name: None,
            selected_category_index: 0,
            tickr_detail_parent: AppView::Tickrs,
            project_summaries: HashMap::new(),
            categories: HashMap::new(),
            worked_range: WorkedRange::Today,
            timeline_range: TimelineRange::Day,
            focus_mode: FocusMode::Content,
            selected_tab_index: 0,
            projects_search_query: String::new(),
            projects_search_active: false,
            edit_popup: None,
            new_category_popup: None,
            new_tickr_popup: None,
        };

        // Initialize categories and project summaries
        app.refresh_categories_for_tickrs();
        app.refresh_project_summaries();

        app
    }

    /// Central update function - process an event and mutate state.
    pub fn update(&mut self, event: AppEvent) {
        match event {
            AppEvent::Tick => {
                if self.running_tickr.is_some() {
                    self.refresh_running_tickrs();
                }
            }
            AppEvent::KeyPress(key) => self.handle_key(key),
        }

        if self.running_tickr.is_some() {
            self.refresh_view_data();
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        if self.edit_popup.is_some() {
            self.handle_edit_key(key);
            return;
        }
        if self.new_category_popup.is_some() {
            self.handle_new_category_key(key);
            return;
        }
        if self.new_tickr_popup.is_some() {
            self.handle_new_tickr_key(key);
            return;
        }
        if self.projects_search_active {
            self.handle_projects_search_key(key);
            return;
        }

        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('h') => {
                self.navigate_to(AppView::Dashboard);
                self.load_dashboard();
            }
            KeyCode::Char('p') => {
                self.navigate_to(AppView::Projects);
                self.load_projects();
            }
            KeyCode::Char('t') => {
                self.navigate_to(AppView::Tickrs);
                self.load_tickrs();
                self.selected_tickr = None;
                self.selected_tickr_project_name = None;
            }
            KeyCode::Char('w') => {
                self.navigate_to(AppView::WorkedProjects);
                self.load_worked_projects();
                self.selected_project = None;
            }
            KeyCode::Char('l') => {
                self.navigate_to(AppView::Timeline);
                self.load_timeline();
            }
            KeyCode::Char('c') => {
                self.navigate_to(AppView::Categories);
                self.load_categories();
            }
            KeyCode::Char('?') => {
                if self.view == AppView::Help {
                    self.go_back();
                } else {
                    self.navigate_to(AppView::Help);
                }
            }
            KeyCode::Char('/') => {
                if self.view == AppView::Projects {
                    self.projects_search_active = true;
                }
            }
            KeyCode::Tab => {
                if self.focus_mode == FocusMode::TabBar {
                    self.focus_mode = FocusMode::Content;
                } else {
                    self.focus_mode = FocusMode::TabBar;
                }
            }
            KeyCode::BackTab => {
                if self.view == AppView::WorkedProjects {
                    self.toggle_worked_range();
                } else if self.view == AppView::Timeline {
                    self.toggle_timeline_range();
                }
            }
            KeyCode::Char('r') => match self.view {
                AppView::Dashboard => self.load_dashboard(),
                AppView::Projects => self.load_projects(),
                AppView::Tickrs => self.load_tickrs(),
                AppView::ProjectTickrs => self.load_project_tickrs(),
                AppView::WorkedProjects => self.load_worked_projects(),
                AppView::Timeline => self.load_timeline(),
                AppView::Categories => self.load_categories(),
                AppView::TickrDetail => self.refresh_tickr_detail(),
                AppView::Help => {}
            },
            KeyCode::Left => {
                if self.focus_mode == FocusMode::TabBar {
                    self.navigate_tab_left();
                }
            }
            KeyCode::Right => {
                if self.focus_mode == FocusMode::TabBar {
                    self.navigate_tab_right();
                }
            }
            KeyCode::Up => {
                if self.focus_mode == FocusMode::Content {
                    self.move_selection_up();
                }
            }
            KeyCode::Down => {
                if self.focus_mode == FocusMode::Content {
                    self.move_selection_down();
                }
            }
            KeyCode::Enter => {
                if self.focus_mode == FocusMode::TabBar {
                    self.activate_selected_tab();
                } else {
                    self.open_selected();
                }
            }
            KeyCode::Char(' ') => self.toggle_tickr(),
            KeyCode::Char('s') => self.stop_running_tickr(),
            KeyCode::Char('g') => self.go_to_project_from_tickr(),
            KeyCode::Esc => self.go_back(),
            KeyCode::Char('e') => self.open_edit_popup(),
            KeyCode::Char('n') => match self.view {
                AppView::Projects | AppView::ProjectTickrs => self.open_new_tickr_popup(),
                AppView::Categories => self.open_new_category_popup(),
                _ => {}
            },
            _ => {}
        }
    }

    fn navigate_to(&mut self, view: AppView) {
        if self.view != view {
            self.view_history.push(self.view.clone());
            self.view = view;
            if self.view != AppView::Projects {
                self.projects_search_active = false;
            }
            self.load_content_for_view();
            // Update selected_tab_index to match the current view
            if let Some(index) = TABS.iter().position(|v| {
                *v == self.view
                    || (self.view == AppView::ProjectTickrs && *v == AppView::Tickrs)
                    || (self.view == AppView::TickrDetail && *v == AppView::Tickrs)
            }) {
                self.selected_tab_index = index;
            }
        }
    }

    fn load_content_for_view(&mut self) {
        match self.view {
            AppView::Dashboard => self.load_dashboard(),
            AppView::Projects => self.load_projects(),
            AppView::Tickrs => self.load_tickrs(),
            AppView::ProjectTickrs => self.load_project_tickrs(),
            AppView::WorkedProjects => self.load_worked_projects(),
            AppView::Timeline => self.load_timeline(),
            AppView::Categories => self.load_categories(),
            AppView::TickrDetail => self.refresh_tickr_detail(),
            AppView::Help => {}
        }
    }

    fn navigate_tab_left(&mut self) {
        if self.selected_tab_index == 0 {
            self.selected_tab_index = TABS.len() - 1;
        } else {
            self.selected_tab_index -= 1;
        }
    }

    fn navigate_tab_right(&mut self) {
        self.selected_tab_index = (self.selected_tab_index + 1) % TABS.len();
    }

    fn activate_selected_tab(&mut self) {
        let target_view = TABS[self.selected_tab_index].clone();
        self.navigate_to(target_view);
        self.focus_mode = FocusMode::Content;
    }

    fn handle_edit_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.edit_popup = None;
                self.clear_status();
            }
            KeyCode::Enter => self.apply_edit_popup(),
            KeyCode::Up => {
                if let Some(popup) = self.edit_popup.as_mut() {
                    popup.select_prev();
                }
            }
            KeyCode::Down => {
                if let Some(popup) = self.edit_popup.as_mut() {
                    popup.select_next();
                }
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if let Some(popup) = self.edit_popup.as_mut() {
                    popup.label.pop();
                }
            }
            KeyCode::Char(ch) => {
                if ch.is_control() {
                    return;
                }
                if let Some(popup) = self.edit_popup.as_mut() {
                    popup.label.push(ch);
                }
            }
            _ => {}
        }
    }

    fn handle_new_category_key(&mut self, key: KeyCode) {
        let Some(popup) = self.new_category_popup.as_mut() else {
            return;
        };
        match key {
            KeyCode::Esc => {
                self.new_category_popup = None;
                self.clear_status();
            }
            KeyCode::Enter => self.apply_new_category_popup(),
            KeyCode::Tab => {
                popup.field = match popup.field {
                    CategoryField::Name => CategoryField::Color,
                    CategoryField::Color => CategoryField::Name,
                };
            }
            KeyCode::Backspace | KeyCode::Delete => match popup.field {
                CategoryField::Name => {
                    popup.name.pop();
                }
                CategoryField::Color => {
                    popup.color.pop();
                }
            },
            KeyCode::Char(ch) => {
                if ch.is_control() {
                    return;
                }
                match popup.field {
                    CategoryField::Name => popup.name.push(ch),
                    CategoryField::Color => popup.color.push(ch),
                }
            }
            _ => {}
        }
    }

    fn handle_new_tickr_key(&mut self, key: KeyCode) {
        let Some(popup) = self.new_tickr_popup.as_mut() else {
            return;
        };
        match key {
            KeyCode::Esc => {
                self.new_tickr_popup = None;
                self.clear_status();
            }
            KeyCode::Enter => self.apply_new_tickr_popup(),
            KeyCode::Tab => {
                popup.field = match popup.field {
                    NewTickrField::Label => NewTickrField::Project,
                    NewTickrField::Project => NewTickrField::Category,
                    NewTickrField::Category => NewTickrField::StartNow,
                    NewTickrField::StartNow => NewTickrField::Label,
                };
            }
            KeyCode::Up => match popup.field {
                NewTickrField::Project => {
                    if !popup.projects.is_empty() {
                        if popup.project_index == 0 {
                            popup.project_index = popup.projects.len() - 1;
                        } else {
                            popup.project_index -= 1;
                        }
                    }
                }
                NewTickrField::Category => {
                    if !popup.categories.is_empty() {
                        if popup.category_index == 0 {
                            popup.category_index = popup.categories.len() - 1;
                        } else {
                            popup.category_index -= 1;
                        }
                    }
                }
                _ => {}
            },
            KeyCode::Down => match popup.field {
                NewTickrField::Project => {
                    if !popup.projects.is_empty() {
                        popup.project_index = (popup.project_index + 1) % popup.projects.len();
                    }
                }
                NewTickrField::Category => {
                    if !popup.categories.is_empty() {
                        popup.category_index = (popup.category_index + 1) % popup.categories.len();
                    }
                }
                _ => {}
            },
            KeyCode::Char(' ') => {
                if popup.field == NewTickrField::StartNow {
                    popup.start_now = !popup.start_now;
                } else if popup.field == NewTickrField::Label {
                    popup.label.push(' ');
                }
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if popup.field == NewTickrField::Label {
                    popup.label.pop();
                }
            }
            KeyCode::Char(ch) => {
                if ch.is_control() {
                    return;
                }
                if popup.field == NewTickrField::Label {
                    popup.label.push(ch);
                }
            }
            _ => {}
        }
    }

    fn refresh_view_data(&mut self) {
        match self.view {
            AppView::Dashboard => self.load_dashboard(),
            AppView::Projects => self.load_projects(),
            AppView::Tickrs => self.load_tickrs(),
            AppView::ProjectTickrs => self.load_project_tickrs(),
            AppView::WorkedProjects => self.load_worked_projects(),
            AppView::Timeline => self.load_timeline(),
            AppView::Categories => self.load_categories(),
            AppView::TickrDetail => self.refresh_tickr_detail(),
            AppView::Help => {}
        }
    }

    fn refresh_running_tickrs(&mut self) {
        if let Ok(tickrs) = db::query_tickr(crate::types::TickrQuery::All, &self.db) {
            self.tickrs = tickrs;
            self.running_tickr = None;
            for tickr in &self.tickrs {
                if tickr
                    .intervals
                    .last()
                    .map(|interval| interval.end_time.is_none())
                    .unwrap_or(false)
                {
                    self.running_tickr = tickr.id;
                    break;
                }
            }
            if self.selected_tickr_index >= self.tickrs.len() {
                self.selected_tickr_index = self.tickrs.len().saturating_sub(1);
            }
            self.refresh_categories_for_tickrs();
        }
    }

    fn clear_status(&mut self) {
        self.status = None;
    }

    fn load_dashboard(&mut self) {
        // Load all data for dashboard view
        self.load_projects();
        self.load_tickrs();
        self.load_categories();
    }

    fn load_projects(&mut self) {
        let result = if self.projects_search_query.trim().is_empty() {
            db::query_projects(&self.db)
        } else {
            db::search_projects_by_name(self.projects_search_query.trim(), &self.db)
        };
        match result {
            Ok(projects) => {
                self.projects = projects;
                self.clear_status();
                if self.selected_project_index >= self.projects.len() {
                    self.selected_project_index = self.projects.len().saturating_sub(1);
                }
                self.refresh_project_summaries();
            }
            Err(err) => {
                self.status = Some(format!("Failed to load projects: {err}"));
            }
        }
    }

    fn handle_projects_search_key(&mut self, key: KeyCode) {
        if self.view != AppView::Projects {
            self.projects_search_active = false;
            return;
        }
        match key {
            KeyCode::Esc => {
                self.projects_search_active = false;
                self.projects_search_query.clear();
                self.load_projects();
            }
            KeyCode::Enter => {
                self.projects_search_active = false;
                self.load_projects();
            }
            KeyCode::Backspace | KeyCode::Delete => {
                self.projects_search_query.pop();
                self.load_projects();
            }
            KeyCode::Char(ch) => {
                if ch.is_control() {
                    return;
                }
                self.projects_search_query.push(ch);
                self.load_projects();
            }
            _ => {}
        }
    }

    fn load_worked_projects(&mut self) {
        let result = match self.worked_range {
            WorkedRange::Today => db::query_project_worked_on_today(&self.db),
            WorkedRange::Week => db::query_project_worked_on_week(&self.db),
        };
        match result {
            Ok(projects) => {
                self.worked_projects = projects;
                self.clear_status();
                if self.selected_worked_project_index >= self.worked_projects.len() {
                    self.selected_worked_project_index =
                        self.worked_projects.len().saturating_sub(1);
                }
            }
            Err(err) => {
                self.status = Some(format!("Failed to load worked projects: {err}"));
            }
        }
    }

    fn load_tickrs(&mut self) {
        match db::query_tickr(crate::types::TickrQuery::All, &self.db) {
            Ok(tickrs) => {
                self.tickrs = tickrs;
                self.clear_status();
                if self.selected_tickr_index >= self.tickrs.len() {
                    self.selected_tickr_index = self.tickrs.len().saturating_sub(1);
                }
                self.refresh_categories_for_tickrs();
            }
            Err(err) => {
                self.status = Some(format!("Failed to load tickrs: {err}"));
            }
        }
    }

    fn load_timeline(&mut self) {
        self.load_tickrs();
    }

    fn load_categories(&mut self) {
        match db::query_categories(&self.db) {
            Ok(mut categories) => {
                categories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                self.categories_list = categories;
                self.clear_status();
                if self.selected_category_index >= self.categories_list.len() {
                    self.selected_category_index = self.categories_list.len().saturating_sub(1);
                }
            }
            Err(err) => {
                self.status = Some(format!("Failed to load categories: {err}"));
            }
        }
    }

    fn load_project_tickrs(&mut self) {
        let Some(project) = &self.selected_project else {
            self.tickrs.clear();
            return;
        };
        let project_id = match project.id {
            Some(id) => id,
            None => return,
        };
        match db::query_tickr(crate::types::TickrQuery::ByProjectId(project_id), &self.db) {
            Ok(tickrs) => {
                self.tickrs = tickrs;
                self.clear_status();
                if self.selected_tickr_index >= self.tickrs.len() {
                    self.selected_tickr_index = self.tickrs.len().saturating_sub(1);
                }
                self.refresh_categories_for_tickrs();
            }
            Err(err) => {
                self.status = Some(format!("Failed to load tickrs: {err}"));
            }
        }
    }

    fn move_selection_up(&mut self) {
        match self.view {
            AppView::Projects => {
                if self.projects.is_empty() {
                    return;
                }
                if self.selected_project_index == 0 {
                    self.selected_project_index = self.projects.len() - 1;
                } else {
                    self.selected_project_index -= 1;
                }
            }
            AppView::Tickrs | AppView::ProjectTickrs => {
                if self.tickrs.is_empty() {
                    return;
                }
                if self.selected_tickr_index == 0 {
                    self.selected_tickr_index = self.tickrs.len() - 1;
                } else {
                    self.selected_tickr_index -= 1;
                }
            }
            AppView::WorkedProjects => {
                if self.worked_projects.is_empty() {
                    return;
                }
                if self.selected_worked_project_index == 0 {
                    self.selected_worked_project_index = self.worked_projects.len() - 1;
                } else {
                    self.selected_worked_project_index -= 1;
                }
            }
            AppView::Categories => {
                if self.categories_list.is_empty() {
                    return;
                }
                if self.selected_category_index == 0 {
                    self.selected_category_index = self.categories_list.len() - 1;
                } else {
                    self.selected_category_index -= 1;
                }
            }
            _ => {}
        }
    }

    fn move_selection_down(&mut self) {
        match self.view {
            AppView::Projects => {
                if self.projects.is_empty() {
                    return;
                }
                self.selected_project_index =
                    (self.selected_project_index + 1) % self.projects.len();
            }
            AppView::Tickrs | AppView::ProjectTickrs => {
                if self.tickrs.is_empty() {
                    return;
                }
                self.selected_tickr_index = (self.selected_tickr_index + 1) % self.tickrs.len();
            }
            AppView::WorkedProjects => {
                if self.worked_projects.is_empty() {
                    return;
                }
                self.selected_worked_project_index =
                    (self.selected_worked_project_index + 1) % self.worked_projects.len();
            }
            AppView::Categories => {
                if self.categories_list.is_empty() {
                    return;
                }
                self.selected_category_index =
                    (self.selected_category_index + 1) % self.categories_list.len();
            }
            _ => {}
        }
    }

    fn open_selected_project(&mut self) {
        if self.view != AppView::Projects || self.projects.is_empty() {
            return;
        }
        let project = self.projects[self.selected_project_index].clone();
        self.selected_project = Some(project);
        self.navigate_to(AppView::ProjectTickrs);
    }

    fn open_selected_worked_project(&mut self) {
        if self.view != AppView::WorkedProjects || self.worked_projects.is_empty() {
            return;
        }
        let project = self.worked_projects[self.selected_worked_project_index].clone();
        let Some(project_id) = project.id else {
            return;
        };
        self.go_to_project_by_id(project_id, None);
    }

    fn open_selected_tickr(&mut self) {
        if !matches!(self.view, AppView::Tickrs | AppView::ProjectTickrs) || self.tickrs.is_empty()
        {
            return;
        }
        let tickr = self.tickrs[self.selected_tickr_index].clone();
        self.selected_tickr_project_name = self.lookup_project_name(tickr.project_id);
        self.selected_tickr = Some(tickr);
        self.tickr_detail_parent = self.view.clone();
        self.navigate_to(AppView::TickrDetail);
    }

    fn open_selected(&mut self) {
        match self.view {
            AppView::Dashboard => {}
            AppView::Projects => self.open_selected_project(),
            AppView::Tickrs | AppView::ProjectTickrs => self.open_selected_tickr(),
            AppView::WorkedProjects => self.open_selected_worked_project(),
            AppView::Categories => {}
            AppView::TickrDetail => {}
            AppView::Timeline => {}
            AppView::Help => {}
        }
    }

    fn open_edit_popup(&mut self) {
        if self.view != AppView::TickrDetail {
            return;
        }
        let Some(tickr) = &self.selected_tickr else {
            self.status = Some("No task selected.".to_string());
            return;
        };
        let Some(tickr_id) = tickr.id else {
            self.status = Some("Selected task has no id.".to_string());
            return;
        };

        let mut categories = match db::query_categories(&self.db) {
            Ok(categories) => categories,
            Err(err) => {
                self.status = Some(format!("Failed to load categories: {err}"));
                return;
            }
        };
        categories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        let mut options = Vec::new();
        options.push(CategoryOption {
            id: None,
            name: "none".to_string(),
            color: None,
        });
        for category in categories {
            options.push(CategoryOption {
                id: Some(category.id),
                name: category.name,
                color: Some(category.color),
            });
        }

        let mut category_index = 0;
        if let Some(current_id) = tickr.category_id {
            if let Some(index) = options.iter().position(|opt| opt.id == Some(current_id)) {
                category_index = index;
            }
        }

        self.edit_popup = Some(EditTickrPopup {
            tickr_id,
            label: tickr.description.clone(),
            category_index,
            categories: options,
        });
    }

    fn open_new_category_popup(&mut self) {
        if self.view != AppView::Categories {
            return;
        }
        self.new_category_popup = Some(NewCategoryPopup {
            name: String::new(),
            color: String::new(),
            field: CategoryField::Name,
        });
    }

    fn open_new_tickr_popup(&mut self) {
        if self.view != AppView::Projects && self.view != AppView::ProjectTickrs {
            return;
        }

        let projects = match db::query_projects(&self.db) {
            Ok(projects) => projects,
            Err(err) => {
                self.status = Some(format!("Failed to load projects: {err}"));
                return;
            }
        };
        let mut project_options = Vec::new();
        for project in projects {
            if let Some(id) = project.id {
                project_options.push(ProjectOption {
                    id,
                    name: project.name,
                });
            }
        }
        if project_options.is_empty() {
            self.status = Some("No projects available.".to_string());
            return;
        }

        let mut categories = match db::query_categories(&self.db) {
            Ok(categories) => categories,
            Err(err) => {
                self.status = Some(format!("Failed to load categories: {err}"));
                return;
            }
        };
        categories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        let mut category_options = Vec::new();
        category_options.push(CategoryOption {
            id: None,
            name: "none".to_string(),
            color: None,
        });
        for category in categories {
            category_options.push(CategoryOption {
                id: Some(category.id),
                name: category.name,
                color: Some(category.color),
            });
        }

        let selected_project_id = match self.view {
            AppView::ProjectTickrs => self.selected_project.as_ref().and_then(|p| p.id),
            AppView::Projects => self
                .projects
                .get(self.selected_project_index)
                .and_then(|project| project.id),
            _ => None,
        };
        let mut project_index = 0;
        if let Some(project_id) = selected_project_id {
            if let Some(index) = project_options.iter().position(|opt| opt.id == project_id) {
                project_index = index;
            }
        }

        self.new_tickr_popup = Some(NewTickrPopup {
            label: String::new(),
            project_index,
            category_index: 0,
            projects: project_options,
            categories: category_options,
            start_now: true,
            field: NewTickrField::Label,
        });
    }

    fn apply_edit_popup(&mut self) {
        let Some(popup) = self.edit_popup.take() else {
            return;
        };

        let category_id = popup
            .categories
            .get(popup.category_index)
            .and_then(|option| option.id);

        if let Err(err) =
            db::update_tickr_details(popup.tickr_id, popup.label.clone(), category_id, &self.db)
        {
            self.status = Some(format!("Failed to update task: {err}"));
            self.edit_popup = Some(popup);
            return;
        }

        self.status = Some("Task updated.".to_string());
        self.refresh_tickr_detail();
        self.refresh_categories_for_tickrs();
        match self.tickr_detail_parent {
            AppView::Tickrs => self.load_tickrs(),
            AppView::ProjectTickrs => self.load_project_tickrs(),
            _ => {}
        }
    }

    fn apply_new_category_popup(&mut self) {
        let Some(popup) = self.new_category_popup.take() else {
            return;
        };
        let name = popup.name.trim().to_string();
        if name.is_empty() {
            self.status = Some("Category name is required.".to_string());
            self.new_category_popup = Some(popup);
            return;
        }

        let color_input = popup.color.trim();
        let color = match normalize_hex_color(color_input) {
            Some(color) => color,
            None => {
                self.status = Some("Color must be a 6-digit hex value.".to_string());
                self.new_category_popup = Some(popup);
                return;
            }
        };

        if let Err(err) = db::create_category(name.clone(), color.clone(), &self.db) {
            self.status = Some(format!("Failed to create category: {err}"));
            self.new_category_popup = Some(popup);
            return;
        }

        self.status = Some("Category created.".to_string());
        self.load_categories();
        if let Some(index) = self
            .categories_list
            .iter()
            .position(|category| category.name == name)
        {
            self.selected_category_index = index;
        }
    }

    fn apply_new_tickr_popup(&mut self) {
        let Some(popup) = self.new_tickr_popup.take() else {
            return;
        };

        let label = popup.label.trim().to_string();
        if label.is_empty() {
            self.status = Some("Task label is required.".to_string());
            self.new_tickr_popup = Some(popup);
            return;
        }

        let project_id = match popup.projects.get(popup.project_index) {
            Some(project) => project.id,
            None => {
                self.status = Some("Project selection is required.".to_string());
                self.new_tickr_popup = Some(popup);
                return;
            }
        };

        let category_id = popup
            .categories
            .get(popup.category_index)
            .and_then(|option| option.id);

        let tickr = Tickr {
            id: None,
            project_id,
            description: label.clone(),
            category_id,
            intervals: Vec::new(),
        };

        let tickr_id = match db::create_tickr(tickr, &self.db) {
            Ok(id) => id,
            Err(err) => {
                self.status = Some(format!("Failed to create task: {err}"));
                self.new_tickr_popup = Some(popup);
                return;
            }
        };

        if popup.start_now {
            if let Some(running_id) = self.running_tickr {
                if let Err(err) = db::end_tickr(running_id, &self.db) {
                    self.status = Some(format!("Failed to stop running task: {err}"));
                    return;
                }
                self.running_tickr = None;
            }
            if let Err(err) = db::start_tickr(tickr_id, &self.db) {
                self.status = Some(format!("Failed to start task: {err}"));
                return;
            }
            self.running_tickr = Some(tickr_id);
            self.status = Some("Task created and started.".to_string());
        } else {
            self.status = Some("Task created.".to_string());
        }

        self.refresh_project_summaries();
        match self.view {
            AppView::Projects => self.load_projects(),
            AppView::ProjectTickrs => self.load_project_tickrs(),
            AppView::Tickrs => self.load_tickrs(),
            _ => {}
        }
    }

    fn go_back(&mut self) {
        if let Some(prev_view) = self.view_history.pop() {
            //Assign the new view manually (cyclic loop when using navigate_to)
            self.view = prev_view;
            self.load_content_for_view();
        }
        self.clear_status();
    }

    fn toggle_tickr(&mut self) {
        let tickr = match self.current_tickr() {
            Some(tickr) => tickr,
            None => {
                self.status = Some("No task selected.".to_string());
                return;
            }
        };
        let Some(id) = tickr.id else {
            return;
        };

        let is_current_running = tickr
            .intervals
            .last()
            .map(|interval| interval.end_time.is_none())
            .unwrap_or(false)
            && tickr.id == self.running_tickr;
        let result = if is_current_running {
            db::end_tickr(id, &self.db)
        } else {
            if self.running_tickr.is_some() {
                // Stop currently running tickr if any
                if let Some(running_id) = self.running_tickr {
                    if let Err(err) = db::end_tickr(running_id, &self.db) {
                        self.status = Some(format!("Failed to stop currently running task: {err}"));
                        return;
                    }
                }
                self.running_tickr = None;
            }
            db::start_tickr(id, &self.db)
        };

        if let Err(err) = result {
            self.status = Some(format!("Failed to update task: {err}"));
            return;
        } else {
            self.running_tickr = Some(id);
        }

        match self.view {
            AppView::Tickrs => self.load_tickrs(),
            AppView::ProjectTickrs => self.load_project_tickrs(),
            AppView::TickrDetail => self.refresh_tickr_detail(),
            _ => {}
        }
    }

    fn refresh_tickr_detail(&mut self) {
        let Some(tickr) = &self.selected_tickr else {
            return;
        };
        let Some(id) = tickr.id else {
            return;
        };
        match db::query_tickr_by_id(id, &self.db) {
            Ok(Some(updated)) => {
                self.selected_tickr = Some(updated);
                self.status = None;
                self.refresh_categories_for_tickrs();
            }
            Ok(None) => {
                self.status = Some("Task not found.".to_string());
            }
            Err(err) => {
                self.status = Some(format!("Failed to refresh task: {err}"));
            }
        }
    }

    fn refresh_categories_for_tickrs(&mut self) {
        let mut missing = HashSet::new();
        for tickr in &self.tickrs {
            if let Some(id) = tickr.category_id {
                if !self.categories.contains_key(&id) {
                    missing.insert(id);
                }
            }
        }
        if let Some(tickr) = &self.selected_tickr {
            if let Some(id) = tickr.category_id {
                if !self.categories.contains_key(&id) {
                    missing.insert(id);
                }
            }
        }

        for id in missing {
            match db::query_category_by_id(id, &self.db) {
                Ok(Some(category)) => {
                    self.categories.insert(id, category);
                }
                Ok(None) => {}
                Err(err) => {
                    self.status = Some(format!("Failed to load categories: {err}"));
                    return;
                }
            }
        }
    }

    pub fn category_for_tickr(&self, tickr: &Tickr) -> Option<&TickrCategory> {
        tickr.category_id.and_then(|id| self.categories.get(&id))
    }

    fn current_tickr(&self) -> Option<&Tickr> {
        match self.view {
            AppView::Tickrs | AppView::ProjectTickrs => self.tickrs.get(self.selected_tickr_index),
            AppView::TickrDetail => self.selected_tickr.as_ref(),
            _ => None,
        }
    }

    fn lookup_project_name(&self, project_id: u32) -> Option<String> {
        db::query_project_by_id(project_id, &self.db)
            .ok()
            .flatten()
            .map(|project| project.name)
    }

    fn go_to_project_from_tickr(&mut self) {
        if self.view != AppView::TickrDetail {
            return;
        }
        let Some(tickr) = &self.selected_tickr else {
            return;
        };
        self.go_to_project_by_id(tickr.project_id, tickr.id);
    }

    fn stop_running_tickr(&mut self) {
        self.refresh_running_tickrs();
        let running = self.tickrs.iter().find(|tickr| {
            tickr
                .intervals
                .last()
                .map(|interval| interval.end_time.is_none())
                .unwrap_or(false)
        });
        let Some(tickr) = running else {
            self.status = Some("No task running.".to_string());
            return;
        };
        let Some(id) = tickr.id else {
            self.status = Some("Running task has no id.".to_string());
            return;
        };

        if let Err(err) = db::end_tickr(id, &self.db) {
            self.status = Some(format!("Failed to stop task: {err}"));
            return;
        }

        self.go_to_project_by_id(tickr.project_id, Some(id));
    }

    fn go_to_project_by_id(&mut self, project_id: u32, highlight_tickr_id: Option<u32>) {
        let project = match db::query_project_by_id(project_id, &self.db) {
            Ok(Some(project)) => project,
            _ => {
                self.status = Some("Project not found.".to_string());
                return;
            }
        };

        self.load_projects();
        if let Some(index) = self
            .projects
            .iter()
            .position(|item| item.id == Some(project_id))
        {
            self.selected_project_index = index;
        }
        self.selected_project = Some(project);
        self.navigate_to(AppView::ProjectTickrs);
        self.load_project_tickrs();
        if let Some(tickr_id) = highlight_tickr_id {
            if let Some(index) = self
                .tickrs
                .iter()
                .position(|item| item.id == Some(tickr_id))
            {
                self.selected_tickr_index = index;
            }
        }
        self.selected_tickr = None;
        self.selected_tickr_project_name = None;
        self.clear_status();
    }

    fn toggle_worked_range(&mut self) {
        self.worked_range = match self.worked_range {
            WorkedRange::Today => WorkedRange::Week,
            WorkedRange::Week => WorkedRange::Today,
        };
        if self.view == AppView::WorkedProjects {
            self.load_worked_projects();
        }
    }

    fn toggle_timeline_range(&mut self) {
        self.timeline_range = match self.timeline_range {
            TimelineRange::Day => TimelineRange::Week,
            TimelineRange::Week => TimelineRange::Day,
        };
        if self.view == AppView::Timeline {
            self.load_timeline();
        }
    }

    pub fn project_summary_for(&self, project: &Project) -> ProjectSummary {
        project
            .id
            .and_then(|id| self.project_summaries.get(&id).copied())
            .unwrap_or_default()
    }

    fn refresh_project_summaries(&mut self) {
        match db::query_tickr(crate::types::TickrQuery::All, &self.db) {
            Ok(tickrs) => {
                let mut summaries: HashMap<ProjectId, ProjectSummary> = HashMap::new();
                for tickr in tickrs {
                    let entry = summaries
                        .entry(tickr.project_id)
                        .or_insert(ProjectSummary::default());
                    let last_interval = tickr.intervals.last();
                    let is_running = last_interval
                        .map(|interval| interval.end_time.is_none())
                        .unwrap_or(false);
                    if is_running || tickr.intervals.is_empty() {
                        entry.open += 1;
                    } else {
                        entry.ended += 1;
                    }
                    for interval in &tickr.intervals {
                        if let Some(end_time) = interval.end_time {
                            let seconds = end_time
                                .signed_duration_since(interval.start_time)
                                .num_seconds();
                            if seconds > 0 {
                                entry.total_seconds += seconds;
                            }
                        }
                    }
                }
                self.project_summaries = summaries;
            }
            Err(err) => {
                self.status = Some(format!("Failed to load project summaries: {err}"));
            }
        }
    }
}

fn normalize_hex_color(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = if let Some(stripped) = trimmed.strip_prefix('#') {
        if stripped.len() == 6 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
            format!("#{}", stripped.to_ascii_uppercase())
        } else {
            return None;
        }
    } else if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        format!("#{}", trimmed.to_ascii_uppercase())
    } else {
        return None;
    };
    Some(normalized)
}
