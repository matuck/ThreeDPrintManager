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

use iced::{Background, Fill, Length, Pixels};
use iced::widget::{button, text, container, Container, row, column, grid, text_input, Text, scrollable, image};
use iced::alignment::{Horizontal};
use iced::widget::text::Alignment;
use rusqlite::fallible_iterator::FallibleIterator;
use crate::{Message, ThreeDPrintManager};

impl ThreeDPrintManager {
    pub fn main_view(&self) -> Container<'_, Message> {
        let main_content = row![self.main_side_panel(), self.main_project_panel()];

        Container::new(main_content).width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
    }
    fn main_side_panel(&self) -> Container<'_, Message> {
        let prog_options = column![]
            .push(
                button(Container::new(Text::new("Settings")).center_x(Fill))
                    .style(Self::rounded_button)
                    .on_press(Message::ToSettingsPage)
                    .width(Length::FillPortion(4))

            )
            .push(
                button(Container::new(Text::new("Scan Project Dirs")).center_x(Fill))
                    .style(Self::rounded_button)
                    .on_press(Message::ScanProjectDirs)
                    .width(Length::FillPortion(4))
            ).width(Length::Fill);
        let mut filter_column = column![].width(Length::Fill).height(Length::Fill);
        filter_column = filter_column
            .push(
                text_input("Search", &self.namefilter)
                    .style(|theme, status| {
                        let mut style = text_input::default(theme, status);
                        style.background = Background::Color(iced::Color::BLACK);
                        style
                    })
                    .on_input(Message::FilterChanged)
            );
        let mut tag_boxes = column![].width(Length::Fill).height(Length::Fill);
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
                column![filter_column].height(Length::Fill).width(Length::Fill)
            )
            .push(
                row![prog_options].width(Length::Fill)
            ).width(Length::Fill);
        Container::new(side_panel).width(Length::Fixed(20.0)).height(Length::Fill).center_x(Length::FillPortion(1)).center_y(Length::Fill)
    }
    fn main_project_panel(&self) -> Container<'_, Message> {
        let mut project_grid = row![].height(Length::Fill).width(Length::Fill);
        let mut project_panel = column![text("Project List").size(50)].height(Length::Fill).width(Length::Fill);

        for project in &self.project_list {
            let project_file = project.get_default_or_first_image_file();
            let imagepath = match project_file {
                Some(project_file) => project_file.get_image_path(),
                None => "".to_string()
            };

            project_grid = project_grid.push(
                button(
                       container(
                           column![
                               text(project.name.to_string()).align_x(Alignment::Center).width(Length::Fill),
                               image(imagepath)
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
        Container::new(project_panel).width(Length::Fill).height(Length::Fill).center_x(Length::FillPortion(4)).center_y(Length::Fill)
    }
}
