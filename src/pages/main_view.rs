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
use env_logger::fmt::style::AnsiColor::White;
use iced::{Background, Fill, Length, Theme};
use iced::widget::{button, text, Container, container, row, column, Column, grid, text_input, Text, scrollable, checkbox};
use crate::{Message, ThreeDPrintManager};

impl ThreeDPrintManager {
    pub fn main_view(&self) -> Container<'_, Message> {
        let main_content = row![self.main_side_panel(), self.main_project_panel()];

        Container::new(main_content).width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill)
    }
    fn main_side_panel(&self) -> Container<'_, Message> {
        let mut prog_options = column![]
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
        let mut side_panel = column![text("Filter").size(50)]
            .push(
                column![filter_column].height(Length::Fill).width(Length::Fill)
            )
            .push(
                row![prog_options].width(Length::Fill)
            ).width(Length::Fill);
        Container::new(side_panel).width(Length::Fixed(20.0)).height(Length::Fill).center_x(Length::FillPortion(1)).center_y(Length::Fill)
    }
    fn main_project_panel(&self) -> Container<'_, Message> {
        let mut project_grid = grid![].height(Length::Fill);
        let mut project_panel = column![text("Project List").size(50)].height(Length::Fill).width(Length::Fill);

        for project in &self.project_list {
            project_grid = project_grid.push(
                //text(project.name.as_str())
                button(project.name.as_str()
                       /*container(
                           row![text(project.name.to_string())]
                               //.align_items(Alignment::Center)
                               .spacing(10),
                       )
                           .width(Length::Fixed(200.0))
                           .height(Length::Fixed(50.0))
                           //.center_x()
                           //.center_y(),*/
                ).style(button::text)
                    .on_press(Message::SelectProject(project.clone()))

            );
        }
        project_panel = project_panel.push(project_grid);
        Container::new(project_panel).width(Length::Fill).height(Length::Fill).center_x(Length::FillPortion(4)).center_y(Length::Fill)
    }
}
