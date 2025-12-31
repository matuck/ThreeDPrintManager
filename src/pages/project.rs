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

use iced::Length;
use iced::widget::{button, text, Container, row, column, container, scrollable};
use crate::models;
use models::file::ProjectFile;
use crate::{Message, ThreeDPrintManager};

impl ThreeDPrintManager {
    pub fn project(&self) -> Container<'_, Message> {
        let mut proj = self.selected_project.clone().unwrap();
        let main_content = iced::widget::column![]
            .push(
                row![
                        column![text(proj.name.to_string()).size(50)].width(Length::Fill),
                        column![
                            button("Open Directory").on_press(Message::OpenDirectory(proj.path)),
                            button("Back").on_press(Message::ToMainPage)
                        ]
                ].width(Length::Fill)
            )
            .push(
                self.project_view_files()
            );
        Container::new(main_content).width(Length::Fill).height(Length::Fill)
    }

    fn project_view_files(&self) -> Container<'_, Message> {
        let mut file_list = column![].width(Length::Fill).height(Length::Fill);
        for file in self.selected_project.clone().unwrap().files.unwrap() {
            let mut strip_path= self.selected_project.clone().unwrap().path;
            strip_path.push_str("/");
            let mut thisrow = row![].width(Length::Fill);
            thisrow = thisrow.push(text!("{}", file.path.to_string().replace(strip_path.as_str(), "")).width(Length::Fill));
            if file.path.contains(".3mf") || file.path.contains(".stl") || file.path.contains(".jpg") || file.path.contains(".jpeg") || file.path.contains(".png") {
                thisrow = thisrow.push(button("Set Default"));
            }
            thisrow = thisrow.push(button("Open").on_press(Message::OpenDirectory(file.path)));
            file_list = file_list.push(thisrow)
        }

        Container::new(scrollable(file_list)).width(Length::Fill).height(Length::Fill)
    }
}