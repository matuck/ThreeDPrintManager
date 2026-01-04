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
use std::{fs, path};
use serde::{Serialize, Deserialize};
use regex::Regex;
use path::{PathBuf, Path};
use fs::create_dir_all;
use std::process::Stdio;
#[allow(unused)]
use log::{error, warn, info, debug, trace};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    pub id: i32,
    pub path: String,
    pub notes: Option<String>,
    pub project_id: i32,
    pub default: bool,
}

impl ProjectFile {
    pub fn get_image_path(&self, stl_thumb_path: String) -> String {
        if self.is_image_type() {
            return self.path.clone();
        }
        if self.can_generate_to_image() {
            return self.get_generated_image_path(stl_thumb_path);
        }
        "".to_string()
    }
    pub fn is_image_type(&self) -> bool {
        let regex = Regex::new(r"((?i)\.png|\.jpg|\.jpeg|\.gif)").unwrap();
        regex.is_match(&self.path)
    }

    pub fn can_generate_to_image(&self) -> bool {
        let regex = Regex::new(r"((?i)\.stl|\.3mf)").unwrap();
        regex.is_match(&self.path)
    }

    pub fn is_text_type(&self) -> bool {
        let regex = Regex::new(r"((?i)\.txt|\.md|\.json|\.toml|\.yaml|\.yml|\.ini)").unwrap();
        regex.is_match(&self.path)
    }

    pub fn is_image_or_can_generate_to_image(&self) -> bool {
        if self.is_image_type() || self.can_generate_to_image() {
            return true;
        }
        false
    }

    pub fn get_generated_image_path(&self, stl_thumb_path: String) -> String {
        //get the path to file source
        let mut path = PathBuf::from(&self.path.clone());
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        //get the project directory
        path = path.parent().unwrap().to_path_buf();
        //set the generated directory
        path.push(".3DManager");
        //ensure generated directory exists
        create_dir_all(&path).unwrap();
        let image_file = format!("{}/{}{}", path.to_str().unwrap(), filename, ".png");
        if !Path::new(&image_file).exists() {
            info!("Creating image file {}", image_file);
            let process = std::process::Command::new(stl_thumb_path)
                .args(&[self.path.clone(), image_file.clone()])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn().unwrap();

            let output = process.wait_with_output().unwrap();

            if output.status.success() {
                return image_file
            } else {
                error!("Error generating image file {}. stl_thumb error  is {}", image_file, String::from_utf8(output.stderr).unwrap_or("".to_string()));
            }
        } else {
            return image_file;
        }

        "".to_string()
    }
}