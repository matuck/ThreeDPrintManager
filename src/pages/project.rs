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
use std::fs::{self, OpenOptions};
use std::io::Write;
use iced::{Element, Length, Theme};
use iced::alignment::Horizontal;
use iced::widget::{button, text, Container, row, Row, column, scrollable, text_editor, text_input, Space, image};
use log::{error, info};
use crate::{ThreeDManager};
use crate::db_manager::DbManager;
use crate::models::file::ProjectFile;
use crate::models::project::Project;
use crate::models::project_tag::ProjectTag;

pub struct ProjectPage {
    stl_thumb: String,
    db_manager: DbManager,
    selected_project: Project,
    project_note_editor: text_editor::Content,
    project_file_note_editor: text_editor::Content,
    tag_to_add: String,
    selected_project_file: Option<ProjectFile>,
    selected_image_project_file: Option<ProjectFile>,
    source_name: String,
    source_url: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackToMain,
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
    ProjectSave,
    SourceNameUpdate(String),
    SourceURLUpdate(String),
    AddSource,
    OpenSource(String),
}

impl ProjectPage {
    pub fn new(project: Project) -> Self {
        let db_manager = ThreeDManager::setup_db_connection();
        let mut project_page = ProjectPage {
            stl_thumb: ThreeDManager::get_stl_thumb(),
            db_manager,
            selected_project: project,
            project_note_editor: text_editor::Content::with_text(""),
            project_file_note_editor: text_editor::Content::with_text(""),
            tag_to_add: "".to_string(),
            selected_project_file: None,
            selected_image_project_file: None,
            source_name: "".to_string(),
            source_url: "".to_string(),
        };
        project_page.project_note_editor = text_editor::Content::with_text(project_page.selected_project.notes.as_str());
        project_page.selected_project_file = project_page.selected_project.get_default_or_first_image_file();
        project_page.selected_image_project_file = project_page.selected_project.get_default_or_first_image_file();
        project_page.update_project_file_note_editor_on_selection();
        project_page
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::BackToMain => {}  //This should not occur as should be handled in main update function

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
            Message::ProjectSave => {
                self.db_manager.update_project(self.selected_project.clone());
            }
            Message::SourceNameUpdate(source_name) => {
                self.source_name = source_name;
            }
            Message::SourceURLUpdate(source_url) => {
                self.source_url = source_url;
            }
            Message::AddSource => {
                self.selected_project = self.db_manager.add_source(self.selected_project.clone(), self.source_name.clone(), self.source_url.clone());
                self.source_name = "".to_string();
                self.source_url = "".to_string();
            }
            Message::OpenSource(source_url) => {
                match open::that_detached(source_url.clone()) {
                    Ok(()) => info!("Opened '{}' successfully.", source_url),
                    Err(err) => error!("An error occurred when opening '{}': {}", source_url, err),
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
    pub fn view(&self) -> Element<'_, Message> {
        let main_content = iced::widget::column![]
            .push(
                row![
                    column![text_input("Project Name", &self.selected_project.name).size(50)
                        .on_input(Message::ProjectNameUpdate),
                    ].width(Length::Fill),
                    column![
                        button(text("Open Directory").align_x(Horizontal::Center)).style(ThreeDManager::rounded_button).on_press(Message::OpenDirectory(self.selected_project.path.clone())),
                        row![
                            button(text("Save").align_x(Horizontal::Center)).style(ThreeDManager::rounded_button).on_press(Message::ProjectSave),
                            button(text("Back").align_x(Horizontal::Center)).style(ThreeDManager::rounded_button).on_press(Message::BackToMain)
                        ]
                    ].align_x(Horizontal::Center),
                ].width(Length::Fill)
            )
            .push(
                row![
                    column![image(self.selected_image_project_file.clone().unwrap().get_image_path(self.stl_thumb.clone()))].height(Length::Fill).width(Length::Fill).height(Length::Fill),
                    column![
                        row![text_editor(&self.project_note_editor)
                                .placeholder("Type something here...")
                                .on_action(Message::ProjectNotesEdit)].height(Length::Fill).width(Length::Fill),
                        row![self.project_view_tags()].width(Length::Fill)
                    ].height(Length::Fill),
                ].width(Length::Fill).height(Length::Fill)
            )
            .push(
                self.project_view_files()
            )
            .push(
                self.project_view_sources()
            );
        Element::new(Container::new(main_content).width(Length::Fill).height(Length::Fill))
    }

    fn project_view_tags(&self) -> Container<'_, Message> {
        let mut content = column![].width(Length::Fill);
        content = content.push(text("Tags:").size(30).width(Length::Fill));
        let mut tag_list = row![].width(Length::Fill);
       //let tags2 = self.selected_project.clone().unwrap().tags.unwrap();
        for tag in self.selected_project.tags.iter() {
            tag_list = tag_list.push(
                button(text(tag.tag.to_string()))
                    .style(ThreeDManager::button_tag_style)
                    .padding(3).on_press(Message::RemoveTag(tag.clone()))
                );
            tag_list = tag_list.push(Space::new().width(5));
        }
        content = content.push(Row::wrap(tag_list));
        let mut add_tag = row![].width(Length::Fill);
        add_tag = add_tag.push(
            text_input("Tag to add", &self.tag_to_add.as_str()).on_input(Message::TagToAddChanged),
        )
            .push(
                button(text("Add Tag")).style(ThreeDManager::rounded_button).on_press(Message::ProjectAddTag)
            );

        content = content.push(add_tag);
        Container::new(content).width(Length::Fill)
    }

    fn project_view_files(&self) -> Container<'_, Message> {
        let mut file_list = column![].width(Length::Fill).height(Length::Fill);
        for file in self.selected_project.files.iter() {
            let mut strip_path= self.selected_project.clone().path;
            strip_path.push_str("/");
            let mut this_row = row![].width(Length::Fill);
            this_row = this_row.push(
                button(
                    text!("{}", file.path.to_string().replace(strip_path.as_str(), "")))
                    .style(|theme :&Theme,status|{
                        let palette = theme.extended_palette();
                        let mut style = button::text(theme, status);
                        match self.selected_project_file.clone() {
                            Some(selected_file) => {
                                if file.id == selected_file.id {
                                    style.background = Some(palette.secondary.strong.color.into());
                                    style.text_color = palette.primary.base.text;
                                }
                            }
                            None => {}
                        }

                        style
                    })
                    .on_press(Message::SelectFile(file.clone()))
                    .width(Length::Fill));
            file_list = file_list.push(this_row)
        }

        let mut file_actions_buttons = row![];
        //open for selected file
        file_actions_buttons = file_actions_buttons.push(
            button(text("Open").align_x(Horizontal::Center))
                .on_press(Message::OpenDirectory(self.selected_project_file.clone().unwrap().path))
                .style(ThreeDManager::rounded_button)
        );
        if self.selected_project_file.clone().unwrap().is_image_or_can_generate_to_image() {
            file_actions_buttons = file_actions_buttons.push(
                button(text("Set Default").align_x(Horizontal::Center))
                    .on_press(Message::SetFileDefault)
                    .style(ThreeDManager::rounded_button)
            );
        }
        let file_list_container = column![
            row![scrollable(file_list)],
            file_actions_buttons.wrap()
        ].width(Length::Fill).height(Length::Fill).align_x(Horizontal::Center);
        let file_note_editor  = column![
                text("File Notes:").size(30).width(Length::Fill),
                text_editor(&self.project_file_note_editor)
                    .placeholder("Type something here...")
                    .on_action(Message::ProjectFileNotesEdit).height(Length::Fill),
                button(text("Save File Notes").align_x(Horizontal::Center).width(Length::Fill)).on_press(Message::ProjectFileSave).width(Length::Fill).style(ThreeDManager::rounded_button),
        ].height(Length::Fill).width(Length::Fill).align_x(Horizontal::Center);
        Container::new(row![file_list_container,file_note_editor]).width(Length::Fill).height(Length::Fill)
    }
    fn project_view_sources(&self) -> Container<'_, Message> {
        let mut content = column![].width(Length::Fill);
        let mut main_content = row![].width(Length::Fill);
        main_content = main_content.push(text("Sources:").size(30));
        for source in self.selected_project.sources.iter() {
            main_content = main_content.push(
                button(text(source.name.clone())).on_press(Message::OpenSource(source.url.clone()))
            )
        }
        let mut add_content = row![].width(Length::Fill);
        add_content = add_content.push(text_input("Source Name", &self.source_name)
            .on_input(Message::SourceNameUpdate));
        add_content = add_content.push(text_input("Source URL", &self.source_url)
            .on_input(Message::SourceURLUpdate));
        add_content = add_content.push(button(text("Add Source")).on_press(Message::AddSource));
        content = content.push(main_content.wrap());
        content = content.push(add_content);
        Container::new(content).width(Length::Fill)
    }
}