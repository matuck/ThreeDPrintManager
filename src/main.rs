/*
 * Copyright (c) 2025-2026 Mitch Tuck
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

pub mod config;
pub mod models;
mod db_manager;
mod pages;

use models::{project::Project, project_tag::ProjectTag};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use config::Config;
use iced::{Element};
use iced::widget::{button, Theme, text_editor};
use iced_dialog::{dialog};
use open;
use which::which;
#[allow(unused)]
use log::{error, warn, info, debug, trace};

use env_logger::Env;
use crate::db_manager::DbManager;
use crate::models::file::ProjectFile;

pub fn main() -> iced::Result {
    let mut default_log_level = "error";
    if cfg!(debug_assertions) {
        default_log_level = "error,ThreeDPrintManager=info";
    }
    let env = Env::default()
        .filter_or("RUST_LOG", default_log_level)
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    iced::application(ThreeDManager::default, ThreeDManager::update, ThreeDManager::view)
        .title(ThreeDManager::title)
        .centered()
        .theme(ThreeDManager::theme)
        .run()
}

enum Screen {
    Main,
    Project,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {

    CloseApplication,
    //Page Switching Messages
    ToMainPage,
    ToSettingsPage,

    //MainPage Messages
    ScanProjectDirs,
    SelectProject(Project),
    FilterChanged(String),
    FilterTagToggle(ProjectTag),

    //Settings Messages
    SetTheme(Theme),
    SettingsCancel,
    SettingsSave,
    SettingsAddProjectDirectory,
    SettingsRemoveProjectDirectory(String),

    //Project Page
    OpenDirectory(String),
    ProjectNotesEdit(text_editor::Action),
    ProjectFileNotesEdit(text_editor::Action),
    RemoveTag(ProjectTag),
    TagToAddChanged(String),
    ProjectAddTag,
    ProjectNameUpdate(String),
    SelectFile(ProjectFile),
    ProjectFileSave,
    SetFileDefault,
}

pub struct ThreeDManager {
    screen: Screen,
    config: Config,
    db_manager: DbManager,
    project_list: Vec<Project>,
    selected_project: Project,
    namefilter: String,
    project_note_editor: text_editor::Content,
    project_file_note_editor: text_editor::Content,
    tag_to_add: String,
    tag_list: Vec<ProjectTag>,
    filter_tags: Vec<ProjectTag>,
    selected_project_file: Option<ProjectFile>,
    selected_image_project_file: Option<ProjectFile>,
    stl_thumb: String,
}

impl ThreeDManager {
    /**
     * Set Application Title
     */
    fn title(&self) -> String {
        let screen = match self.screen {
            Screen::Main => "main",
            Screen::Project => "project",
            Screen::Settings => "settings",
        };
        format!("{screen} - Iced")
    }
    fn rounded_button(theme: &Theme, status: button::Status) -> button::Style {
        let mut style = button::primary(theme, status);
        style.border.radius = iced::border::radius(20);
        style
    }
    /**
     * Process Messages
     */
    fn update(&mut self, message: Message) {
        match message {
            Message::CloseApplication => {
                error!("You need to have stl-thumb installed first.");
                std::process::exit(1);
            }
            //Page Switching Messages

            Message::ToMainPage => {
                self.get_projects();
                self.selected_project = Project::default();
                self.screen = Screen::Main;
            }
            Message::ToSettingsPage => {
                self.screen = Screen::Settings;
            }

            //Settings messages
            Message::SetTheme(theme) => {
                self.config.set_theme(theme);
            }
            Message::SettingsCancel => {
                self.get_projects();
                self.config = Config::default();
                self.screen = Screen::Main;
            }
            Message::SettingsSave => {
                self.get_projects();
                self.config.save();
                self.screen = Screen::Main;
            }
            Message::SettingsAddProjectDirectory => {
                self.add_project_directory();
            }
            Message::SettingsRemoveProjectDirectory(path) => {
                self.config.remove_print_path(path.as_str());
            }

            //main page
            Message::ScanProjectDirs => {
                self.scan_project_dirs();
                self.get_projects();
            }
            Message::SelectProject(mut project) => {
                self.db_manager.update_project_files(project.clone(),  project.get_file_system_files());
                self.selected_project = self.db_manager.get_project(project.id);
                self.project_note_editor = text_editor::Content::with_text(self.selected_project.notes.as_str());
                self.selected_project_file = self.selected_project.get_default_or_first_image_file();
                self.update_project_file_note_editor_on_selection();
                self.selected_image_project_file = self.selected_project.get_default_or_first_image_file();
                self.screen = Screen::Project;
            }
            Message::FilterChanged(filter) => {
               self.namefilter = filter;
                self.get_projects();
            }
            Message::FilterTagToggle(tag) => {
                if let Some(pos) = self.filter_tags.iter().position(|x| *x == tag) {
                    self.filter_tags.remove(pos);
                } else {
                    self.filter_tags.push(tag);
                }
                self.get_projects();
            }

            //Project Page
            Message::OpenDirectory(directory) => {
                match open::that_detached(directory.clone()) {
                    Ok(()) => info!("Opened '{}' successfully.", directory),
                    Err(err) => error!("An error occurred when opening '{}': {}", directory, err),
                }
            }
            Message::ProjectNotesEdit(project_note) => {
                self.project_note_editor.perform(project_note);
                self.selected_project.notes = self.project_note_editor.text();
            }
            Message::ProjectFileNotesEdit(project_file_note) => {
                self.project_file_note_editor.perform(project_file_note);
            }
            Message::ProjectFileSave => {
                let mut current_project_file = self.selected_project_file.clone().unwrap();
                let file_note = self.project_file_note_editor.text();
                if current_project_file.is_text_type() {
                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(current_project_file.path).unwrap();
                    let _ = file.write_all(file_note.as_bytes());
                } else {
                    current_project_file.notes = Some(file_note);
                    let new_project_file = self.db_manager.update_project_file(current_project_file);
                    self.selected_project = self.db_manager.get_project(new_project_file.project_id);
                    self.selected_project_file = Some(new_project_file);
                    self.update_project_file_note_editor_on_selection();
                }
            }
            Message::SetFileDefault => {
                let mut file = self.selected_project_file.clone().unwrap();
                file.default = true;
                self.selected_project_file = Some(self.db_manager.update_project_file(file));
            }
            Message::RemoveTag(tag) => {
                self.selected_project = self.db_manager.project_remove_tag(self.selected_project.clone(), tag);
            }
            Message::TagToAddChanged(tag) => {
                self.tag_to_add = tag;
            }
            Message::ProjectAddTag => {
                self.selected_project = self.db_manager.project_add_tag(self.selected_project.clone(), self.tag_to_add.clone());
                self.tag_to_add = "".to_string();
            }
            Message::ProjectNameUpdate(project_name) => {
                self.selected_project.name = project_name;
            }
            Message::SelectFile(file) => {
                self.selected_project_file = Some(file.clone());
                self.update_project_file_note_editor_on_selection();
                if file.is_image_or_can_generate_to_image() {
                    self.selected_image_project_file = Some(file.clone());
                }
            }
        }

    }
    pub fn update_project_file_note_editor_on_selection(&mut self) {
        self.project_file_note_editor = match self.selected_project_file.clone() {
            Some(project_file) => {
                if project_file.is_text_type() {
                    let file_contents = fs::read_to_string(&project_file.path).unwrap_or("".to_string());
                    text_editor::Content::with_text(file_contents.as_str())
                } else {
                    text_editor::Content::with_text(project_file.notes.unwrap_or("".to_string()).as_str())
                }
            },
            None => text_editor::Content::with_text("")
        };
    }
    fn scan_project_dirs(&mut self) {
        if self.config.print_paths.is_none() { return ()}
        let print_paths = self.config.print_paths.clone().unwrap();
        for project_dir in print_paths.iter() {
            self.scan_project_dir(project_dir.clone());
        }
    }
    fn scan_project_dir(&mut self, project_dir: String) {
        for entry in fs::read_dir(Path::new(project_dir.as_str())).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                if !self.does_project_with_path_exist(entry.path().to_str().unwrap().to_string()) {
                    let mut project  = self.create_project(
                        entry.file_name().to_str().unwrap().to_string(),
                        entry.path().to_str().unwrap().to_string(),
                        "".to_string()
                    );
                    self.db_manager.update_project_files(project.clone(),  project.get_file_system_files());
                }
                debug!("Scanning Project directory {}. The Project Name is {}", entry.path().display(), entry.file_name().display());
            }
        }
    }

    fn create_project (&mut self, project_name: String, project_path: String, project_notes: String) -> Project {
        let new_project = Project {
            id: 0,
            path: project_path,
            name: project_name,
            notes: project_notes,
            tags: vec![],
            files: vec![],
            sources: vec![],
        };
        self.db_manager.create_project(new_project).unwrap()
    }
    fn does_project_with_path_exist(&mut self, project_path: String) -> bool {
        let project_list = self.db_manager.get_filtered_projects(None,Some(project_path),None);
        if project_list.len() > 0 {
            return true
        }
        false
    }
    fn get_projects(&mut self) {
        let mut optionfilter = None;
        if !self.namefilter.eq(&"".to_string()) {
            optionfilter = Some(self.namefilter.clone());
        }
        let mut filter_tags :Option<Vec<ProjectTag>> = None;
        if self.filter_tags.len() > 0 {
            filter_tags = Some(self.filter_tags.clone());
        }
        self.project_list = self.db_manager.get_filtered_projects(optionfilter,None,filter_tags);
        info!("There are {} projects", self.project_list.len());
    }
    /*fn get_project_files(&mut self) {

    }*/
    /**
     * Pick and render view
     */
    fn view(&self) -> Element<'_, Message> {
        let mut color = iced::Color::TRANSPARENT;
        if cfg!(debug_assertions) {
            color = iced::Color::BLACK;
        }
        match self.screen {
            Screen::Main => {
                if self.stl_thumb.eq("") {
                    let dialog_content = "Please install stl-thumb first";
                    dialog(true, Element::new(self.main_view()).explain(color), dialog_content)
                        .title("Save")
                        .push_button(iced_dialog::button("OK", Message::CloseApplication))
                        .width(350)
                        .height(234)
                        .into()
                } else {
                    let dialog_content = "Please add print project directories in the settings page.";
                    dialog(self.config.clone().print_path_empty_or_none(), Element::new(self.main_view()).explain(color), dialog_content)
                        .title("Save")
                        .push_button(iced_dialog::button("OK", Message::ToSettingsPage))
                        .width(350)
                        .height(234)
                        .into()
                }
            },
            Screen::Project => {
                Element::new(self.project()).explain(color)
            },
            Screen::Settings => {
                Element::new(self.settings()).explain(color)
            },
        }
    }

    /**
     * Gets the theme to be used.
     * Matches from self.config.theme
     */
    fn theme(&self) -> Theme {
        self.config.get_theme()
    }
}
impl Default for ThreeDManager {
    fn default() -> Self {
        let stl_thumb = which("stl-thumb").unwrap_or(PathBuf::default()).to_str().unwrap_or("").to_string();
        info!("ThreeDManager Started");
        let config = Config::default();
        let mut dbfile = Config::get_config_dir().unwrap();
        dbfile.push("3DManager.db");
        let dbmgr = db_manager::DbManager::new(dbfile.to_str().unwrap().to_string());
        dbmgr.run_migration();
        let tag_list = dbmgr.get_tag_list();
        let mut myself = Self {
            screen: Screen::Main,
            config,
            db_manager: dbmgr,
            project_list: vec![],
            selected_project: Project::default(),
            namefilter: "".to_string(),
            project_note_editor: text_editor::Content::with_text(""),
            project_file_note_editor: text_editor::Content::with_text(""),
            tag_to_add: "".to_string(),
            tag_list,
            filter_tags: Vec::new(),
            selected_project_file: None,
            selected_image_project_file: None,
            stl_thumb
        };
        myself.get_projects();
        return myself;
    }
}