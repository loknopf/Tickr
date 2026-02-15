use std::collections::{HashMap, HashSet};

use crossterm::event::KeyCode;
use rusqlite::Connection;

use crate::db;
use crate::types::{CategoryId, Project, ProjectId, Tickr, TickrCategory, TickrId};

use super::{AppEvent, AppView, ProjectSummary, WorkedRange};

/// The top-level application state.
pub struct App {
    pub running: bool,
    pub running_tickr: Option<TickrId>,
    pub db: Connection,
    pub view: AppView,
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
    pub edit_popup: Option<EditTickrPopup>,
    pub new_category_popup: Option<NewCategoryPopup>,
}

#[derive(Clone, Debug)]
pub struct CategoryOption {
    pub id: Option<CategoryId>,
    pub name: String,
    pub color: Option<String>,
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
        let running_tickr = match tickrs.iter().find(|tickr| {
                tickr
                    .intervals
                    .last()
                    .map(|interval| interval.end_time.is_none())
                    .unwrap_or(false)
            }).and_then(|tickr| tickr.id) {
                Some(id) => Some(id),
                None => None,
            };
        Self {
            running: true,
            running_tickr,
            db,
            view: AppView::Projects,
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
            edit_popup: None,
            new_category_popup: None,
        }
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

        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('p') => {
                self.view = AppView::Projects;
                self.load_projects();
            }
            KeyCode::Char('t') => {
                self.view = AppView::Tickrs;
                self.load_tickrs();
                self.selected_tickr = None;
                self.selected_tickr_project_name = None;
            }
            KeyCode::Char('w') => {
                self.view = AppView::WorkedProjects;
                self.load_worked_projects();
                self.selected_project = None;
            }
            KeyCode::Char('c') => {
                self.view = AppView::Categories;
                self.load_categories();
            }
            KeyCode::Tab => {
                self.toggle_worked_range();
            }
            KeyCode::Char('r') => match self.view {
                AppView::Projects => self.load_projects(),
                AppView::Tickrs => self.load_tickrs(),
                AppView::ProjectTickrs => self.load_project_tickrs(),
                AppView::WorkedProjects => self.load_worked_projects(),
                AppView::Categories => self.load_categories(),
                AppView::TickrDetail => self.refresh_tickr_detail(),
            },
            KeyCode::Up => self.move_selection_up(),
            KeyCode::Down => self.move_selection_down(),
            KeyCode::Enter => self.open_selected(),
            KeyCode::Char(' ') => self.toggle_tickr(),
            KeyCode::Char('s') => self.stop_running_tickr(),
            KeyCode::Char('g') => self.go_to_project_from_tickr(),
            KeyCode::Esc => self.go_back(),
            KeyCode::Char('e') => self.open_edit_popup(),
            KeyCode::Char('n') => self.open_new_category_popup(),
            _ => {}
        }
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

    fn refresh_view_data(&mut self) {
        match self.view {
            AppView::Projects => self.load_projects(),
            AppView::Tickrs => self.load_tickrs(),
            AppView::ProjectTickrs => self.load_project_tickrs(),
            AppView::WorkedProjects => self.load_worked_projects(),
            AppView::Categories => self.load_categories(),
            AppView::TickrDetail => self.refresh_tickr_detail(),
        }
    }

    fn refresh_running_tickrs(&mut self) {
        if let Ok(tickrs) = db::query_tickr(crate::types::TickrQuery::All, &self.db) {
            self.tickrs = tickrs;
            self.running_tickr = None;
            for tickr in &self.tickrs {
                if tickr.intervals.last().map(|interval| interval.end_time.is_none()).unwrap_or(false) {
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

    fn load_projects(&mut self) {
        match db::query_projects(&self.db) {
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

    fn load_categories(&mut self) {
        match db::query_categories(&self.db) {
            Ok(mut categories) => {
                categories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                self.categories_list = categories;
                self.clear_status();
                if self.selected_category_index >= self.categories_list.len() {
                    self.selected_category_index =
                        self.categories_list.len().saturating_sub(1);
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
                self.selected_project_index = (self.selected_project_index + 1) % self.projects.len();
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
        self.view = AppView::ProjectTickrs;
        self.load_project_tickrs();
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
        if !matches!(self.view, AppView::Tickrs | AppView::ProjectTickrs) || self.tickrs.is_empty() {
            return;
        }
        let tickr = self.tickrs[self.selected_tickr_index].clone();
        self.selected_tickr_project_name = self.lookup_project_name(tickr.project_id);
        self.selected_tickr = Some(tickr);
        self.tickr_detail_parent = self.view.clone();
        self.view = AppView::TickrDetail;
    }

    fn open_selected(&mut self) {
        match self.view {
            AppView::Projects => self.open_selected_project(),
            AppView::Tickrs | AppView::ProjectTickrs => self.open_selected_tickr(),
            AppView::WorkedProjects => self.open_selected_worked_project(),
            AppView::Categories => {},
            AppView::TickrDetail => {}
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

    fn apply_edit_popup(&mut self) {
        let Some(popup) = self.edit_popup.take() else {
            return;
        };

        let category_id = popup
            .categories
            .get(popup.category_index)
            .and_then(|option| option.id);

        if let Err(err) = db::update_tickr_details(
            popup.tickr_id,
            popup.label.clone(),
            category_id,
            &self.db,
        ) {
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

    fn go_back(&mut self) {
        match self.view {
            AppView::TickrDetail => {
                self.view = self.tickr_detail_parent.clone();
                self.selected_tickr = None;
                self.selected_tickr_project_name = None;
            }
            AppView::ProjectTickrs => {
                self.view = AppView::Projects;
                self.selected_project = None;
            }
            AppView::WorkedProjects => {
                self.view = AppView::Projects;
            }
            AppView::Categories => {
                self.view = AppView::Projects;
            }
            _ => {}
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
            .unwrap_or(false) && tickr.id == self.running_tickr;
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
        }else{
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
        tickr
            .category_id
            .and_then(|id| self.categories.get(&id))
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
        self.view = AppView::ProjectTickrs;
        self.load_project_tickrs();
        if let Some(tickr_id) = highlight_tickr_id {
            if let Some(index) = self.tickrs.iter().position(|item| item.id == Some(tickr_id)) {
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
