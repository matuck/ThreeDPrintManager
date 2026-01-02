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

use iced::alignment::Horizontal;
use iced::Length;
use iced::widget::{row, Column, Container, button, Space, column, text, pick_list};
use iced::Theme;
use rfd::FileDialog;
use crate::{Message, ThreeDManager};

impl ThreeDManager {
    pub fn settings(&self) -> Container<'_, Message> {
        let mut project_dirs_widget :Column<Message> = Column::new();
        if self.config.print_paths.is_some() {
            for directory in self.config.print_paths.clone().unwrap() {
                project_dirs_widget = project_dirs_widget.push(
                    row![
                        button("Delete").on_press(Message::SettingsRemoveProjectDirectory(directory.clone())),
                        Space::new().width(20),
                        column![text(directory.to_string())].width(Length::Fill),
                    ]
                );
            }
        }
        let main_content = iced::widget::column![text("Settings").size(50)]
            .push(
                row![
                    text("Theme:"),
                    Space::new().width(30),
                    pick_list(Theme::ALL,Some(self.theme()), Message::SetTheme) ,
                ].width(Length::Fill)
            )
            .push(
                iced::widget::column![
                    text("Project Directories:").size(40),
                    project_dirs_widget,
                    row![column![button("Add Directory").on_press(Message::SettingsAddProjectDirectory)].width(Length::Fill).align_x(Horizontal::Right)].width(Length::Fill)
                ].width(Length::Fill)
            ).width(Length::Fill).height(Length::Fill);
        let action_content = iced::widget::column![
                row![
                    button("Cancel").on_press(Message::SettingsCancel),
                    Space::new().width(30),
                    button("Save").on_press(Message::SettingsSave)
                ]
            ].width(Length::Fill).align_x(Horizontal::Right);
        Container::new(iced::widget::column![main_content,action_content]).width(Length::Fill).height(Length::Fill)
    }

    pub fn add_project_directory (&mut self) {
        let files = FileDialog::new()
            .set_directory("/")
            .pick_folder();
        if files.is_some() {
            self.config.add_print_path(files.unwrap().to_str().unwrap());
        }
    }
}