/*
    potato_plant_replay - controls replaying a given mission. clients connect to a websocket and recieve data
    Copyright (C) 2022  Bailey Danyluk

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use std::fs;
use std::io::Read;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use hyper::{
    StatusCode, Body,
    http::{Response}
};

use log::{warn, debug};

static STATIC_404_PAGE: &str = "<!DOCTYPE html>
<html>
    <head>
        <title>404 Not Found</title>
    </head>
    <body>
        <h1>404</h1>
    </body>
</html>";

pub enum StaticFileStorage {
    Memory(Vec<u8>),
    Disk(&'static str)
}

pub enum StaticFile {
    HTML(StaticFileStorage),
    JavaScript(StaticFileStorage)
}

impl StaticFile {
    fn get_vec_data(storage: &StaticFileStorage, root_path: &Path) -> Result<Vec<u8>, std::io::Error> {
        match storage {
            StaticFileStorage::Memory(data) => Ok(data.to_vec()),
            StaticFileStorage::Disk(path) => fs::read(root_path.join(path))
        }
    }

    fn serve_html(storage: &StaticFileStorage, root_path: &Path) -> Result<Response<Body>, std::io::Error> {
        Ok(
            Response::builder()
                .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                .body(Body::from(StaticFile::get_vec_data(storage, root_path)?))
            .unwrap()
        )
    }

    fn serve_js(storage: &StaticFileStorage, root_path: &Path) -> Result<Response<Body>, std::io::Error> {
        Ok(
            Response::builder()
                .status(StatusCode::OK)
                    .header("Content-Type", "application/javascript")
                .body(Body::from(StaticFile::get_vec_data(storage, root_path)?))
            .unwrap()
        )
    }

    fn serve(&self, root_path: &Path) -> Result<Response<Body>, std::io::Error> {
        let response = match self {
            StaticFile::HTML(storage_type) => StaticFile::serve_html(storage_type, root_path),
            StaticFile::JavaScript(storage_type) => StaticFile::serve_js(storage_type, root_path)
        };

        response
    }
}

pub struct StaticServer {
    root_path: PathBuf,
    static_files: HashMap<PathBuf, StaticFile>,
    static_404: StaticFile
}

impl StaticServer {
    pub fn new(root_file_offset: impl Into<PathBuf>) -> StaticServer {
        StaticServer {
            root_path: root_file_offset.into(),
            static_files: HashMap::new(),
            static_404: StaticFile::HTML(StaticFileStorage::Memory(STATIC_404_PAGE.as_bytes().to_vec()))
        }
    }

    pub fn register(&mut self, path: impl Into<PathBuf>, file: StaticFile) {
        let true_path = path.into();
        if self.static_files.contains_key(&true_path) {
            warn!("Static file already exists at path {:?}", true_path.into_os_string());
            return;
        }

        debug!(target: "StaticServer", "Registering file at {:?}", true_path);
        self.static_files.insert(true_path, file);
    }

    pub fn serve_404(&self) -> Response<Body> {
        let mut response = self.static_404.serve(&self.root_path).unwrap();
        *response.status_mut() = StatusCode::NOT_FOUND;
        response
    }

    pub fn serve(&self, path: impl Into<PathBuf>) -> Response<Body> {
        let true_path = path.into();
        
        let file = self.static_files.get(&true_path);
        if let None = file {
            warn!(target: "StaticServer", "Attempting to GET file that has not been registered: {:?}", true_path.into_os_string());
            return self.serve_404()
        }

        let response = file.unwrap().serve(&self.root_path);
        if let Err(e) = response {
            warn!(target: "StaticServer", "Cannot serve file {:?} because {:?}", true_path.into_os_string(), e.kind());
            return self.serve_404()
        }

        response.unwrap()
    }
}

