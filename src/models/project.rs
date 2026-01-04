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
use std::fs;
use std::path::Path;
use log::debug;
use crate::models;
use serde::{Serialize, Deserialize};
use models::{file::ProjectFile, project_tag::ProjectTag, project_source::ProjectSource};
use regex::Regex;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub notes: String,
    pub files: Vec<ProjectFile>,
    pub tags: Vec<ProjectTag>,
    pub sources: Vec<ProjectSource>,
}

impl Project {
    pub fn get_file_system_files(&mut self) -> Vec<String> {
        Project::scan_dir(self.path.clone())
    }
    fn scan_dir(dir: String) -> Vec<String> {
        let mut result  :Vec<String> = Vec::new();
        debug!("Scanning Directory: {}", dir);
        for entry in fs::read_dir(Path::new(dir.as_str())).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() && !entry.path().to_str().unwrap().contains(".3DManager") {
                let mut subresult = Project::scan_dir(entry.path().to_str().unwrap().to_string());
                result.append(&mut subresult);
                debug!("Scanning Project directory {}. The Project Name is {}", entry.path().display(), entry.file_name().display());
            } else {
                if !entry.path().to_str().unwrap().contains(".3DManager") {
                    result.push(entry.path().to_str().unwrap().to_string());
                }
            }
        }
        result
    }
}

impl Project {
    pub fn get_default_or_first_image_file(&self) -> Option<ProjectFile> {
        let default_files :Vec<ProjectFile>= self.files.clone().into_iter().filter(|file| file.default).collect();
        if default_files.len() > 0 {
            return Some(default_files.first().unwrap().clone());
        }
        let regex = Regex::new(r"((?i)\.png|\.jpg|\.jpeg|\.gif|\.stl|\.3mf)").unwrap();
        let filtered_files :Vec<ProjectFile> = self.files.clone().into_iter().filter(|file| regex.is_match(file.path.as_str())).collect();
        if filtered_files.len() > 0 {
            return Some(filtered_files.first().unwrap().clone());
        }
        None
    }
}
impl Default for Project {
    fn default() -> Self {
        Project {
            id: 0,
            name: "".to_string(),
            path: "".to_string(),
            notes: "".to_string(),
            files: vec![],
            tags: vec![],
            sources: vec![],
        }
    }
}