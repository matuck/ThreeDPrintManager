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

use std::path::{PathBuf};
use config::Config;
use iced::{Element};
use iced::widget::{button, Theme};
use which::which;
#[allow(unused)]
use log::{error, warn, info, debug, trace};

use env_logger::Env;
use crate::db_manager::DbManager;
use crate::pages::{main_view, project, settings};
pub fn main() -> iced::Result {
    let mut default_log_level = "error";
    if cfg!(debug_assertions) {
        default_log_level = "error,3DManager=info";
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
    Main(main_view::MainView),
    Project(project::ProjectPage),
    Settings(settings::SettingsPage),
}

#[derive(Debug, Clone)]
pub enum Message {
    MainPage(main_view::Message),
    ProjectPage(project::Message),
    SettingsPage(settings::Message),
}

pub struct ThreeDManager {
    screen: Screen,
    config: Config,
}

impl ThreeDManager {
    /**
     * Set Application Title
     */
    fn title(&self) -> String {
        let screen = match self.screen {
            Screen::Main(_)=> "Project List",
            Screen::Project(_) => "Project",
            Screen::Settings(_) => "Settings",
        };
        format!("3D Manager - {screen}")
    }
    fn rounded_button(theme: &Theme, status: button::Status) -> button::Style {
        let mut style = button::primary(theme, status);
        style.border.radius = iced::border::radius(20);
        style
    }
    fn button_tag_style(theme: &Theme, status :button::Status) -> button::Style {
        let mut style = button::primary(theme, status);
        let palette = theme.extended_palette();
        style.border.radius = iced::border::right(20);
        style.background = Some(palette.success.strong.color.into());
        style
    }
    /**
     * Process Messages
     */
    fn update(&mut self, message: Message) {
        match message {
            Message::MainPage(msg) => {
                match msg {
                    main_view::Message::SelectProject(project) => {
                        self.screen = Screen::Project(project::ProjectPage::new(project));
                    }
                    main_view::Message::ToSettingsPage => {
                        self.screen = Screen::Settings(settings::SettingsPage::new(self.config.clone()));
                    }
                    _ => {
                        let Screen::Main(page) = &mut self.screen else { return () };
                        page.update(msg);
                    }
                }
            }
            Message::ProjectPage(msg) => {
                match msg {
                    project::Message::BackToMain => {
                        self.screen = Screen::Main(main_view::MainView::new(self.config.clone()));
                    }
                    _ => {
                        let Screen::Project(page) = &mut self.screen else { return () };
                        page.update(msg);
                    }
                }
            }
            Message::SettingsPage(msg) => {
                match msg {
                    settings::Message::SetTheme(theme) => {
                        self.config.set_theme(theme.clone());
                        //Get settings screen
                        let Screen::Settings(page) = &mut self.screen else { return () };
                        //and give it back the message
                        page.update(settings::Message::SetTheme(theme));

                    }
                    settings::Message::BackToMain(save) => {
                        if save {
                            let Screen::Settings(page) = &mut self.screen else { return () };
                            //and give it back the message
                            page.save_config();
                        }
                        self.config = Config::default();
                        self.screen = Screen::Main(main_view::MainView::new(self.config.clone()));
                    }
                    _ => {
                        //Get settings screen
                        let Screen::Settings(page) = &mut self.screen else { return () };
                        //and give it back the message
                        page.update(msg);
                    }
                }
            }
        }

    }



    /**
     * Pick and render view
     */
    fn view(&self) -> Element<'_, Message> {
        let mut color = iced::Color::TRANSPARENT;
        if cfg!(debug_assertions) {
            color = iced::Color::BLACK;
        }
        let screen = match &self.screen {
            Screen::Main(main_page) => main_page.view().map(Message::MainPage),
            Screen::Project(project_page)=> project_page.view().map(Message::ProjectPage),
            Screen::Settings(settings_page) => settings_page.view().map(Message::SettingsPage),
        };
        screen.explain(color)
    }

    /**
     * Gets the theme to be used.
     * Matches from self.config.theme
     */
    fn theme(&self) -> Theme {
        self.config.get_theme()
    }
    pub fn setup_db_connection() -> DbManager {
        let mut db_file = Config::get_config_dir().unwrap();
        db_file.push("3DManager.db");
        DbManager::new(db_file.to_str().unwrap().to_string())
    }
    pub fn get_stl_thumb() -> String {
        which("stl-thumb").unwrap_or(PathBuf::default()).to_str().unwrap_or("").to_string()
    }
}
impl Default for ThreeDManager {
    fn default() -> Self {
        info!("ThreeDManager Started");
        let config = Config::default();
        let db_mgr = Self::setup_db_connection();
        db_mgr.run_migration();
        Self {
            screen: Screen::Main(main_view::MainView::new(config.clone())),
            config,
        }
    }
}