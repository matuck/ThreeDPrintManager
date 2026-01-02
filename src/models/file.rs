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
extern crate stl_thumb;
use stl_thumb::config::{AAMethod, Config as StlThumbConfig};
use regex::Regex;
use path::{PathBuf, Path};
use fs::create_dir_all;
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
    pub fn get_image_path(&self) -> String {
        if self.is_image_type() {
            return self.path.clone();
        }
        if self.can_generate_to_image() {
            return self.get_generated_image_path();
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

    pub fn is_image_or_can_generate_to_image(&self) -> bool {
        if self.is_image_type() || self.can_generate_to_image() {
            return true;
        }
        false
    }

    pub fn get_generated_image_path(&self) -> String {
        let mut path = PathBuf::from(&self.path.clone());
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        path = path.parent().unwrap().to_path_buf();
        path.push(".3DPrintManager");
        create_dir_all(&path).unwrap();
        let mut stl_thumb_config = StlThumbConfig::default();
        stl_thumb_config.visible = false;
        stl_thumb_config.verbosity = 3;
        stl_thumb_config.model_filename=self.path.clone();
        stl_thumb_config.img_filename=format!("{}/{}{}", path.to_str().unwrap(), filename, ".png");
        let filepath = Path::new(stl_thumb_config.img_filename.as_str());
        if !filepath.exists() {
            info!("Creating file {:?}", stl_thumb_config.img_filename);
            if let Err(e) = stl_thumb::render_to_file(&stl_thumb_config) {
                error!("Application error: {}", e);
                return "".to_string();
            }
        }
        stl_thumb_config.img_filename
    }
}