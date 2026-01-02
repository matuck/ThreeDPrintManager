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

use iced::{Length, Theme};
use iced::widget::{button, text, Container, row, Row, column, scrollable, text_editor, text_input, Space, image};
use crate::{Message, ThreeDManager};

impl ThreeDManager {
    pub fn project(&self) -> Container<'_, Message> {
        let main_content = iced::widget::column![]
            .push(
                row![
                    column![text_input("Project Name", &self.selected_project.name).size(50)
                        .on_input(Message::ProjectNameUpdate),].width(Length::Fill),
                    column![
                        button("Open Directory").on_press(Message::OpenDirectory(self.selected_project.path.clone())),
                        button("Back").on_press(Message::ToMainPage)
                    ],
                ].width(Length::Fill)
            )
            .push(
                row![
                    column![image(self.selected_image_project_file.clone().unwrap().get_image_path(self.stl_thumb.clone()))].height(Length::Fill).width(Length::Fill).height(Length::Fill),
                    //column![text!("placeholder")].height(Length::Fill).width(Length::Fill).height(Length::Fill),
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
            );
        Container::new(main_content).width(Length::Fill).height(Length::Fill)
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
    fn button_tag_style(theme: &Theme, status :button::Status) -> button::Style {
        let mut style = button::primary(theme, status);
        let palette = theme.extended_palette();
        style.border.radius = iced::border::right(20);
        style.background = Some(palette.success.strong.color.into());
        style
    }
    fn project_view_files(&self) -> Container<'_, Message> {
        let mut file_list = column![].width(Length::Fill).height(Length::Fill);
        for file in self.selected_project.files.iter() {
            let mut strip_path= self.selected_project.clone().path;
            strip_path.push_str("/");
            let mut thisrow = row![].width(Length::Fill);
            thisrow = thisrow.push(
                button(
                    text!("{}", file.path.to_string().replace(strip_path.as_str(), "")))
                    .style(button::text)
                    .on_press(Message::SelectFile(file.clone()))
                    .width(Length::Fill));
            if file.path.contains(".3mf") || file.path.contains(".stl") || file.path.contains(".jpg") || file.path.contains(".jpeg") || file.path.contains(".png") {
                thisrow = thisrow.push(button("Set Default"));
            }
            thisrow = thisrow.push(button("Open").on_press(Message::OpenDirectory(file.path.clone())));
            file_list = file_list.push(thisrow)
        }

        Container::new(scrollable(file_list)).width(Length::Fill).height(Length::Fill)
    }
}