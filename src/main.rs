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

use models::{project::Project, project_tag::ProjectTag, file::File};
use std::fs::{self};
use std::path::{Path};
use config::Config;
use iced::{Element};
use iced::widget::{button, Theme};
use iced_dialog::{dialog};
use open;
#[allow(unused)]
use log::{error, warn, info, debug, trace};

use env_logger::Env;
use crate::db_manager::DbManager;

pub fn main() -> iced::Result {
    let mut default_log_level = "error";
    if cfg!(debug_assertions) {
        default_log_level = "error,ThreeDPrintManager=info";
    }
    let env = Env::default()
        .filter_or("RUST_LOG", default_log_level)
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    iced::application(ThreeDPrintManager::default, ThreeDPrintManager::update, ThreeDPrintManager::view)
        .title(ThreeDPrintManager::title)
        .centered()
        .theme(ThreeDPrintManager::theme)
        .run()
}

enum Screen {
    Main,
    Project,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    //Page Switching Messages
    ToMainPage,
    ToSettingsPage,

    //MainPage Messages
    ScanProjectDirs,
    SelectProject(Project),
    FilterChanged(String),

    //Settings Messages
    SetTheme(Theme),
    SettingsCancel,
    SettingsSave,
    SettingsAddProjectDirectory,
    SettingsRemoveProjectDirectory(String),

    //Project Page
    OpenDirectory(String)
}

pub struct ThreeDPrintManager {
    screen: Screen,
    config: Config,
    db_manager: DbManager,
    project_list: Vec<Project>,
    selected_project: Option<Project>,
    namefilter: String,
}

impl ThreeDPrintManager {
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
            //Page Switching Messages

            Message::ToMainPage => {
                self.get_projects();
                self.selected_project = None;
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
                self.selected_project = Some(self.db_manager.get_project(project.id.unwrap()));
                self.screen = Screen::Project;
            }
            Message::FilterChanged(filter) => {
               self.namefilter = filter;
                self.get_projects();
            }

            //Project Page
            Message::OpenDirectory(directory) => {
                match open::that_detached(directory.clone()) {
                    Ok(()) => info!("Opened '{}' successfully.", directory),
                    Err(err) => error!("An error occurred when opening '{}': {}", directory, err),
                }
            }
        }

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
                    self.create_project(
                        entry.file_name().to_str().unwrap().to_string(),
                        entry.path().to_str().unwrap().to_string(),
                        None
                    );
                }
                debug!("Scanning Project directory {}. The Project Name is {}", entry.path().display(), entry.file_name().display());
            }
        }
    }

    fn create_project (&mut self, project_name: String, project_path: String, project_notes: Option<String>) -> Project {
        let new_project = Project {
            id: None,
            path: project_path,
            name: project_name,
            notes: project_notes,
            tags: None,
            files: None,
        };
        self.db_manager.create_project(new_project).unwrap()
    }
    fn does_project_with_path_exist(&mut self, project_path: String) -> bool {
        let project_list = self.db_manager.get_filtered_projects(None,Some(project_path),None).unwrap_or(Vec::new());
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
        self.project_list = self.db_manager.get_filtered_projects(optionfilter,None,None).unwrap();
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
                let dialog_content = "Please add print project directories in the settings page.";
                dialog(self.config.clone().print_path_empty_or_none(), Element::new(self.main_view()).explain(color), dialog_content)
                    .title("Save")
                    .push_button(iced_dialog::button("OK", Message::ToSettingsPage))
                    .width(350)
                    .height(234)
                    .into()
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
impl Default for ThreeDPrintManager {
    fn default() -> Self {
        info!("ThreeDPrintManager Started");
        let config = Config::default();
        let mut dbfile = Config::get_config_dir().unwrap();
        dbfile.push("3DPrintManager.db");
        let dbmgr = db_manager::DbManager::new(dbfile.to_str().unwrap().to_string());
        dbmgr.run_migration();
        let mut myself = Self {
            screen: Screen::Main,
            config,
            db_manager: dbmgr,
            project_list: vec![],
            selected_project: None,
            namefilter: "".to_string(),
        };
        myself.get_projects();
        return myself;
    }
}