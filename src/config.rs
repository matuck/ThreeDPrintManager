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
//pub mod config {
use std::{fs};
use serde::{Serialize, Deserialize};
extern crate serde;
extern crate dirs;
use toml;
use fs::{create_dir_all, write, read_to_string};
use std::path::PathBuf;
use std::string::ToString;
use iced::Theme;


    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Config {
        pub theme: Option<String>,
        pub print_paths: Option<Vec<String>>,
    }

    impl Config {
        pub fn save(&self) {
            let config_output = toml::to_string(self).unwrap_or("".to_string());
            let mut config_file = dirs::config_dir().unwrap().to_path_buf();
            config_file.push("ThreeDPrintManager");
            config_file.push("config.toml");
            write(config_file.as_path(), config_output).unwrap();
        }
        pub fn set_theme(&mut self, theme: Theme) {
            self.theme = Some(theme.to_string());
        }
        pub fn get_theme(&self) -> Theme {
            match self.theme.clone().unwrap_or("".to_string()).as_str() {
                "Light" => Theme::Light,
                "Dark" => Theme::Dark,
                "Dracula" => Theme::Dracula,
                "Nord" => Theme::Nord,
                "Solarized Light" => Theme::SolarizedLight,
                "Solarized Dark" => Theme::SolarizedDark,
                "Gruvbox Light" => Theme::GruvboxLight,
                "Gruvbox Dark" => Theme::GruvboxDark,
                "Catppuccin Latte" => Theme::CatppuccinLatte,
                "Catppuccin Frappé" => Theme::CatppuccinFrappe,
                "Catppuccin Macchiato" => Theme::CatppuccinMacchiato,
                "Catppuccin Mocha" => Theme::CatppuccinMocha,
                "Tokyo Night" => Theme::TokyoNight,
                "Tokyo Night Storm" => Theme::TokyoNightStorm,
                "Tokyo Night Light" => Theme::TokyoNightLight,
                "Kanagawa Wave" => Theme::KanagawaWave,
                "Kanagawa Dragon" => Theme::KanagawaDragon,
                "Kanagawa Lotus" => Theme::KanagawaLotus,
                "Moonfly" => Theme::Moonfly,
                "Nightfly" => Theme::Nightfly,
                "Oxocarbon" => Theme::Oxocarbon,
                "Ferra" => Theme::Ferra,
                _ => Theme::Light,
            }
        }
        pub fn add_print_path(&mut self, path: &str) {
            if self.print_paths.is_some() {
                //println!("{}", files.unwrap().to_str().unwrap());
                self.print_paths.as_mut().unwrap().push(path.to_string());
            } else {
                self.print_paths = vec![path.to_string()].into()
            }
        }
        pub fn remove_print_path(&mut self, path: &str) {
            if self.print_paths.is_some() {
                self.print_paths.as_mut().unwrap().retain(|value| *value != path.to_string());
            }
        }
        pub fn print_path_empty_or_none(&mut self) -> bool {
            if self.print_paths.is_none() { return true; }
            if self.print_paths.as_ref().unwrap().is_empty() {
                return true;
            }
            false
        }
        pub fn get_config_dir() -> Option<PathBuf> {
            let mut config_dir = dirs::config_dir().unwrap().to_path_buf();
            config_dir.push("ThreeDPrintManager");
            Some(config_dir)
        }
    }

    impl Default for Config {
        fn default() -> Self {
            //Set config directory and ensure it exist
            let config_dir = Config::get_config_dir().unwrap();
            create_dir_all(config_dir.as_path()).unwrap();

            let mut config_file = config_dir.clone();
            config_file.push("config.toml");

            let mut config_str = "".to_string();

            if config_file.exists() {
                config_str = read_to_string(config_file).ok().unwrap();
            }
            let mut config = toml::from_str::<Config>(&config_str).ok().unwrap();

            if config.theme.is_none() {
                config.theme = "Nord".to_string().into();
            }

            config
        }
    }
//}