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

use rusqlite::{params, Connection, Result};
use rust_embed::{Embed};
#[allow(unused)]
use log::{error, warn, info, debug, trace};
use crate::models;
use models::project::Project;
use models::project_tag::ProjectTag;
use models::file::ProjectFile;


pub struct DbManager {
    connection: Connection,
}
#[derive(Embed)]
#[folder = "migrations/"]
struct Migrations;

impl DbManager {
    pub fn new(connection_string: String) -> DbManager {
        let conn = Connection::open(connection_string).unwrap();
        let _ = conn.execute(
            "create table if not exists _migrations (version VARCHAR(50) NOT NULL, run_on TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL)",
            params![]
        );
        DbManager { connection: conn }

    }
    pub fn run_migration(&self) {
        let currversion = self.connection.query_one("SELECT * FROM _migrations ORDER BY version DESC LIMIT 1", params![], |row| {
            Ok(row.get::<usize, String>(0)?)
        }).unwrap_or("0".to_string()).parse::<i64>().unwrap();
        debug!("currversion: {}", currversion);
        for file in Migrations::iter() {
            let file_parts = file.split("/").collect::<Vec<&str>>();
            if currversion < file_parts[0].parse::<i64>().unwrap() && file_parts[1] == "up.sql" {
                info!("Running migration {}", file_parts[0]);
                let currfile = Migrations::get(&file.to_string()).unwrap();
                let sql_to_run = std::str::from_utf8(&currfile.data).unwrap();
                let _ = self.connection.execute_batch(sql_to_run);
                let _ = self.connection.execute("INSERT INTO _migrations (version) VALUES (?)", &[&file_parts[0]]);
            }
        }
    }

    pub fn get_project(&self, id: i32) -> Project {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, path, notes FROM projects where id = ?1",
        ).unwrap();

        let mut project = stmt.query_one([id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                notes: row.get(3)?,
                tags: vec![],
                files: vec![],
            })
        }).unwrap();
        let mut files_stmt = self.connection.prepare(
            "SELECT id, path, notes, project_id FROM project_files WHERE project_id = ?1 ORDER BY path",
        ).unwrap();
        let files :Vec<ProjectFile> = files_stmt.query_map([id], |row| {
            Ok(ProjectFile {
                id: row.get(0).unwrap(),
                path: row.get(1).unwrap(),
                notes: row.get(2).unwrap(),
                project_id: row.get(3).unwrap(),
            })
        }).unwrap().into_iter().map(|r| r.unwrap()).collect();
        let mut tags_stmt = self.connection.prepare(
            "SELECT t.id, t.tag FROM projects_tags pt LEFT JOIN tags t on pt.tag_id = t.id WHERE pt.project_id = ?1",
        ).unwrap();
        let tags :Vec<ProjectTag> = tags_stmt.query_map([id], |row| {
            Ok(ProjectTag{
                id: row.get(0).unwrap(),
                tag: row.get(1).unwrap(),
            })
        }).unwrap().into_iter().map(|r| r.unwrap()).collect();
        project.files = files;
        project.tags = tags;
        project
    }

    pub fn get_filtered_projects(&self, name: Option<String>, path: Option<String>, tags: Option<Vec<ProjectTag>>) -> Result<Vec<Project>> {
        let mut sql = "select p.* from projects p".to_string();
        //add joins if needed
        if tags.is_some() {
            sql.push_str(" JOIN projects_tags pt ON pt.project_id = p.id");
        }
        if tags.is_some() || path.is_some() || name.is_some() {
            sql.push_str(" WHERE");
        }
        if name.is_some() {
            sql.push_str(format!(" name LIKE '%{}%'", name.clone().unwrap()).as_str());
        }
        if path.is_some() {
            if name.is_some() {
                sql.push_str(" AND");
            }
            sql.push_str(format!(" path = '{}'", path.clone().unwrap()).as_str());
        }

        if tags.is_some() {
            if path.is_some() || name.is_some() {
                sql.push_str(" AND");
            }
            let mytags = tags.unwrap();
            let mytag_ids :Vec<String>= mytags.iter().map(|tag| tag.id.to_string()).collect();
            sql.push_str(format!(" pt.tag_id IN ({}) GROUP BY p.id HAVING COUNT(DISTINCT pt.tag_id) = {}", mytag_ids.join(",").to_string(), mytags.len()).as_str());
        }
        sql.push_str(" ORDER BY p.name");
        debug!("{}", sql);
        let mut stmt = self.connection.prepare(sql.as_str(),)?;
        let project_rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                notes: row.get(3)?,
                tags: vec![],
                files: vec![],
            })
        })?;

        let result: Vec<Project> = project_rows.into_iter().map(|r| r.unwrap()).collect();
        Ok(result)
    }

    pub fn create_project(&self, project: Project) -> Result<Project> {
        self.connection.execute(
            "INSERT INTO projects (name, path, notes) VALUES (?1, ?2, ?3)", params![project.name, project.path, project.notes],
        )?;
        let last_id = i32::try_from(self.connection.last_insert_rowid()).unwrap();

        Ok(self.get_project(last_id))
    }

    pub fn update_project_files(&self, project: Project, file_system_files: Vec<String>) {
        //get existing files for project
        let mut stmt = self.connection.prepare(
            "SELECT path FROM project_files pf where project_id = ?1",
        ).unwrap();

        let files_query_results = stmt.query_map([project.id], |row| {
            row.get::<usize, String>(0)
        }).unwrap();
        let existing_files: Vec<String> = files_query_results.into_iter().map(|r| r.unwrap()).collect();
        //files_query_results.
        let files_to_add: Vec<_> = file_system_files.clone().into_iter().filter(|item| !existing_files.contains(item)).collect();
        let files_to_delete: Vec<_> = existing_files.clone().into_iter().filter(|item| !file_system_files.contains(item)).collect();
        let mut add_files_stmt = self.connection.prepare(
            "INSERT INTO project_files (project_id, path) VALUES (?1, ?2)",
        ).unwrap();
        for path in files_to_add.clone() {
            let _ = add_files_stmt.execute((project.id, path));
        };
        let mut delete_files_stmt = self.connection.prepare(
            "DELETE FROM project_files WHERE project_id = ?1 AND path = ?2;",
        ).unwrap();
        for path in files_to_delete.clone() {
            let _ = delete_files_stmt.execute((project.id, path));
        };
        info!("{} added files: {:?}", project.name, files_to_add);
        info!("{} deleted files: {:?}", project.name, files_to_delete);
    }
    pub fn project_remove_tag(&self, project: Project, tag: ProjectTag) -> Project {
        let mut stmt = self.connection.prepare(
            "DELETE FROM projects_tags WHERE project_id = ?1 AND tag_id = ?2",
        ).unwrap();

        stmt.execute(params![project.id, tag.id]).unwrap();
        self.get_project(project.id)
    }
    pub fn project_add_tag(&self, project: Project, tag: String) -> Project {
        let mut mytag = self.get_tag_by_tag(tag.clone());
        if mytag.is_err() {
            mytag = self.add_tag(tag.clone());
        }
        let mytag2 = mytag.unwrap();
        let mut proj_have_tag_stmt = self.connection.prepare(
            "SELECT count(*) FROM projects_tags WHERE project_id = ?1 AND tag_id = ?2",
        ).unwrap();
        let tagcount :Result<i32> = proj_have_tag_stmt.query_one([project.id, mytag2.id], | row | {
            Ok(row.get(0)?)
        });
        if tagcount.unwrap() == 0 {
            let mut stmt = self.connection.prepare(
                "INSERT INTO projects_tags (project_id, tag_id) VALUES (?1, ?2)",
            ).unwrap();
            let _ =stmt.execute(params![project.id, mytag2.id]);
        }
        self.get_project(project.id)
    }

    pub fn get_tag_by_tag(&self, tag: String) -> Result<ProjectTag> {
        let mut stmt = self.connection.prepare(
            "Select id, tag FROM tags WHERE tag = ?1 LIMIT 1",
        ).unwrap();
        let mut mytag = stmt.query_one([tag.clone()], |row| {
            Ok(ProjectTag {
                id: row.get(0)?,
                tag: row.get(1)?,
            })
        });
        mytag
    }
    pub fn get_tag_by_id(&self, id: i32) -> Result<ProjectTag> {
        let mut stmt = self.connection.prepare(
            "Select id, tag FROM tags WHERE id = ?1 LIMIT 1",
        ).unwrap();
        let mut mytag = stmt.query_one([id], |row| {
            Ok(ProjectTag {
                id: row.get(0)?,
                tag: row.get(1)?,
            })
        });
        mytag
    }
    pub fn add_tag(&self, tag: String) -> Result<ProjectTag> {
        let mut addstmt = self.connection.prepare(
            "INSERT INTO tags (tag) VALUES (?1)"
        ).unwrap();
        addstmt.execute([tag.clone()]);
        let last_id = i32::try_from(self.connection.last_insert_rowid()).unwrap();
        self.get_tag_by_id(last_id)
    }

    pub fn get_tag_list(&self) -> Vec<ProjectTag> {
        let mut stmt = self.connection.prepare(
            "Select id, tag FROM tags ORDER BY tag",
        ).unwrap();
        stmt.query_map([], |row| {
            Ok(ProjectTag{
                id: row.get(0)?,
                tag: row.get(1)?,
            })
        }).unwrap().into_iter().map(|r| r.unwrap()).collect()
    }
}