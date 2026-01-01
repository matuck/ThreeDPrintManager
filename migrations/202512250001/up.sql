-- Your SQL goes here
CREATE TABLE projects (
   id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL ,
   name VARCHAR NOT NULL,
   path varchar NOT NULL,
   notes text
);
CREATE TABLE tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL ,
  tag VARCHAR NOT NULL
);

CREATE TABLE projects_tags (
   project_id INTEGER REFERENCES projects(id) NOT NULL,
   tag_id INTEGER REFERENCES tags(id) NOT NULL,
   PRIMARY KEY(project_id, tag_id)
);

CREATE TABLE project_files (
   id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
   path VARCHAR NOT NULL,
   notes TEXT,
   isdefault INTEGER NOT NULL DEFAULT 0,
   project_id INTEGER NOT NULL REFERENCES projects(id)
);

CREATE TABLE project_sources (
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  url VARCHAR NOT NULL,
  project_id INTEGER NOT NULL REFERENCES projects(id)
);