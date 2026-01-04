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
use std::fs;
use std::path::Path;
use iced::{Background, Fill, Length, Element};
use iced::widget::{button, text, container, Container, row, column, text_input, Text, scrollable, image};
use iced::alignment::{Horizontal};
use iced::widget::text::Alignment;
use iced_dialog::dialog;
use log::{debug, error, info};
use crate::{ThreeDManager};
use crate::config::Config;
use crate::db_manager::DbManager;
use crate::models::project::Project;
use crate::models::project_tag::ProjectTag;

pub struct MainView {
    config: Config,
    db_manager: DbManager,
    project_list: Vec<Project>,
    name_filter: String,
    tag_list: Vec<ProjectTag>,
    filter_tags: Vec<ProjectTag>,
    stl_thumb: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    CloseApplication,
    ToSettingsPage,
    ScanProjectDirs,
    FilterChanged(String),
    FilterTagToggle(ProjectTag),
    SelectProject(Project),
}
impl MainView {
    pub fn new(config: Config) -> Self {
        let db_manager = ThreeDManager::setup_db_connection();
        let mut main_view = MainView {
            config,
            db_manager,
            project_list: vec![],
            name_filter: "".to_string(),
            tag_list: vec![],
            filter_tags: vec![],
            stl_thumb: ThreeDManager::get_stl_thumb(),
        };
        main_view.get_projects();

        main_view
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::CloseApplication => {
                error!("You need to have stl-thumb installed first.");
                std::process::exit(1);
            }
            Message::ToSettingsPage => {}  //should never get here this is handled in main update
            Message::SelectProject(_) => {} //should never get here this is handled in main update
            Message::ScanProjectDirs => {
                self.scan_project_dirs();
                self.get_projects();
            }
            Message::FilterChanged(filter) => {
                self.name_filter = filter;
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
        }

    }
    pub fn view(&self) -> Element<'_, Message> {
        let main_content = row![self.main_side_panel(), self.main_project_panel()];

        if ThreeDManager::get_stl_thumb().eq("") {
            let dialog_content = "Please install stl-thumb first";
            dialog(true, Element::new(main_content), dialog_content)
                .title("Save")
                .push_button(iced_dialog::button("OK", Message::CloseApplication))
                .width(350)
                .height(234)
                .into()
        } else {
            let dialog_content = "Please add print project directories in the settings page.";
            dialog(self.config.clone().print_path_empty_or_none(), Element::new(main_content), dialog_content)
                .title("Save")
                .push_button(iced_dialog::button("OK", Message::ToSettingsPage))
                .width(350)
                .height(234)
                .into()
        }
    }
    fn main_side_panel(&self) -> Container<'_, Message> {
        let prog_options = column![]
            .push(
                button(Container::new(Text::new("Settings")).center_x(Fill))
                    .style(ThreeDManager::rounded_button)
                    .on_press(Message::ToSettingsPage)
                    .width(Length::FillPortion(4))

            )
            .push(
                button(Container::new(Text::new("Scan Project Dirs")).center_x(Fill))
                    .style(ThreeDManager::rounded_button)
                    .on_press(Message::ScanProjectDirs)
                    .width(Length::FillPortion(4))
            )
            .width(Fill);
        let mut filter_column = column![].width(Fill).height(Fill);
        filter_column = filter_column
            .push(
                text_input("Search", &self.name_filter)
                    .style(|theme, status| {
                        let mut style = text_input::default(theme, status);
                        style.background = Background::Color(iced::Color::BLACK);
                        style
                    })
                    .on_input(Message::FilterChanged)
            );
        let mut tag_boxes = column![].width(Fill).height(Fill);
        for tag in self.tag_list.iter() {
            if self.filter_tags.contains(&tag) {
                tag_boxes = tag_boxes.push(
                    button(text!("☑ {}", tag.tag)).style(button::text).on_press(Message::FilterTagToggle(tag.clone()))
                );
            } else {
                tag_boxes = tag_boxes.push(
                    button(text!("☐ {}", tag.tag)).style(button::text).on_press(Message::FilterTagToggle(tag.clone()))
                );
            }
        }
        filter_column = filter_column.push(scrollable(tag_boxes));
        let side_panel = column![text("Filter").size(50)]
            .push(
                column![filter_column].height(Fill).width(Fill)
            )
            .push(
                row![prog_options].width(Fill)
            ).width(Fill);
        Container::new(side_panel).width(Length::Fixed(20.0)).height(Fill).center_x(Length::FillPortion(1)).center_y(Fill)
    }
    fn main_project_panel(&self) -> Container<'_, Message> {
        let mut project_grid = row![].height(Fill).width(Fill);
        let mut project_panel = column![text("Project List").size(50)].height(Fill).width(Fill);

        for project in &self.project_list {
            let project_file = project.get_default_or_first_image_file();
            let image_path = match project_file {
                Some(project_file) => project_file.get_image_path(self.stl_thumb.clone()),
                None => "".to_string()
            };

            project_grid = project_grid.push(
                button(
                       container(
                           column![
                               text(project.name.to_string()).align_x(Alignment::Center).width(Fill),
                               image(image_path)
                           ],
                       )
                           .align_x(Horizontal::Center)
                )
                    .on_press(Message::SelectProject(project.clone()))
                    .height(Length::Fixed(200.0))
                    .width(Length::Fixed(200.0))
                    .style(button::text)

            );
        }
        project_panel = project_panel.push(scrollable(project_grid.wrap()));
        Container::new(project_panel).width(Fill).height(Fill).center_x(Length::FillPortion(4)).center_y(Fill)
    }

    fn get_projects(&mut self) {
        let mut option_filter = None;
        if !self.name_filter.eq(&"".to_string()) {
            option_filter = Some(self.name_filter.clone());
        }
        let mut filter_tags :Option<Vec<ProjectTag>> = None;
        if self.filter_tags.len() > 0 {
            filter_tags = Some(self.filter_tags.clone());
        }
        self.project_list = self.db_manager.get_filtered_projects(option_filter, None, filter_tags);
        info!("There are {} projects", self.project_list.len());
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
}
