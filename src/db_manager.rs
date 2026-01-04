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
use crate::models::project_source::ProjectSource;

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
                sources: vec![],
            })
        }).unwrap();

        project.files = self.project_get_files(project.id);
        project.tags = self.project_get_tags(project.id);
        project.sources = self.project_get_sources(project.id);
        project
    }

    pub fn get_filtered_projects(&self, name: Option<String>, path: Option<String>, tags: Option<Vec<ProjectTag>>) -> Vec<Project> {
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
        let mut stmt = self.connection.prepare(sql.as_str(),).unwrap();
        let projects :Vec<Project> = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                notes: row.get(3)?,
                tags: vec![],
                files: vec![],
                sources: vec![],
            })
        }).unwrap().into_iter().map(|r| r.unwrap()).collect();

        /*for mut project in &projects.into_iter() {
            project.files = self.project_get_files(project.id);
            project.tags = self.project_get_tags(project.id);
            project.sources = self.project_get_sources(project.id);
        }*/
        let myprojects = projects.iter().map(|p| {
            let mut proj = p.clone();
            proj.sources = self.project_get_sources(proj.id);
            proj.files = self.project_get_files(proj.id);
            proj.tags = self.project_get_tags(proj.id);
            proj
        }).collect();
        myprojects
    }

    pub fn project_get_files(&self, project_id: i32) -> Vec<ProjectFile> {
        let mut files_stmt = self.connection.prepare(
            "SELECT id, path, notes, project_id, isdefault FROM project_files WHERE project_id = ?1 ORDER BY path",
        ).unwrap();
        let files :Vec<ProjectFile> = files_stmt.query_map([project_id], |row| {
            Ok(ProjectFile {
                id: row.get(0).unwrap(),
                path: row.get(1).unwrap(),
                notes: row.get(2).unwrap(),
                project_id: row.get(3).unwrap(),
                default: row.get(4).unwrap(),
            })
        }).unwrap().into_iter().map(|r| r.unwrap()).collect();
        files
    }

    pub fn project_get_tags(&self, project_id: i32) -> Vec<ProjectTag> {
        let mut tags_stmt = self.connection.prepare(
            "SELECT t.id, t.tag FROM projects_tags pt LEFT JOIN tags t on pt.tag_id = t.id WHERE pt.project_id = ?1",
        ).unwrap();
        let tags :Vec<ProjectTag> = tags_stmt.query_map([project_id], |row| {
            Ok(ProjectTag{
                id: row.get(0).unwrap(),
                tag: row.get(1).unwrap(),
            })
        }).unwrap().into_iter().map(|r| r.unwrap()).collect();
        tags
    }

    pub fn project_get_sources(&self, project_id: i32) -> Vec<ProjectSource> {
        let sources_stmt = self.connection.prepare(
            "SELECT id, url, project_id, name FROM project_sources WHERE project_id = ?1"
        );
        let sources :Vec<ProjectSource> = sources_stmt.unwrap().query_map([project_id], |row| {
            Ok(ProjectSource{
                id: row.get(0)?,
                url: row.get(1)?,
                project_id: row.get(2)?,
                name: row.get(3)?,
            })
        }).unwrap().into_iter().map(|r| r.unwrap()).collect();
        sources
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
        )?;
        let mytag = stmt.query_one([tag.clone()], |row| {
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
        let mytag = stmt.query_one([id], |row| {
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
        addstmt.execute([tag.clone()])?;
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

    pub fn update_project_file(&self, project_file:ProjectFile) -> ProjectFile {
        let mut updatestmt = self.connection.prepare(
            "UPDATE project_files SET path = ?1, notes = ?2, isdefault = ?3,  project_id=?4 WHERE id = ?5;",
        ).unwrap();

        //make all other files not default for project if this file is default.
        if project_file.default {
            let set_not_default_stmt = self.connection.prepare(
                "UPDATE project_files SET isdefault = 0 WHERE project_id = ?1"
            );
            let _ = set_not_default_stmt.unwrap().execute(params![project_file.project_id]);
        }

        let isdefault = match project_file.default {
            true => 1,
            false => 0,
        }.to_string();
        let _ = updatestmt.execute([project_file.path, project_file.notes.unwrap_or("".to_string()),isdefault, project_file.project_id.to_string(), project_file.id.to_string()]);
        self.get_project_file_by_id(project_file.id)
    }
    pub fn get_project_file_by_id(&self, id: i32) -> ProjectFile {
        let mut files_stmt = self.connection.prepare(
            "SELECT id, path, notes, project_id, isdefault FROM project_files WHERE id = ?1 LIMIT 1",
        ).unwrap();
        let file :ProjectFile = files_stmt.query_one([id], |row| {
            Ok(ProjectFile {
                id: row.get(0).unwrap(),
                path: row.get(1).unwrap(),
                notes: row.get(2).unwrap(),
                project_id: row.get(3).unwrap(),
                default: row.get(4).unwrap(),
            })
        }).unwrap();
        file
    }
    pub fn update_project(&self, project: Project) -> Project {
        let mut stmt = self.connection.prepare(
            "UPDATE projects SET name = ?1, notes = ?2, path = ?3 WHERE id = ?4",
        ).unwrap();
        let _ = stmt.execute([project.name, project.notes, project.path, project.id.to_string()]);
        self.get_project(project.id)
    }
    pub fn add_source(&self, project: Project, name: String, url: String) -> Project {
        let mut stmt = self.connection.prepare(
            "INSERT INTO project_sources (name, url, project_id) VALUES (?1, ?2, ?3)",
        ).unwrap();
        let _ = stmt.execute([name, url, project.id.to_string()]);
        self.get_project(project.id)
    }
}