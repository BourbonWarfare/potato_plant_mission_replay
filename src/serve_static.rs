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
use hyper::{
    StatusCode, Body,
    http::{Response}
};

use log::warn;

static STATIC_404: &str = "<!DOCTYPE html>
<html>
    <head>
        <title>404 Not Found</title>
    </head>
    <body>
        <h1>404</h1>
    </body>
</html>";

pub fn serve_404() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(STATIC_404))
    .unwrap()
}

pub fn serve_html(path: impl Into<PathBuf>) -> Response<Body> {
    let path_converted = path.into();
    let file = match fs::read(&path_converted) {
        Ok(f) => f,
        Err(err) => {
            warn!(target: "serve_static.serve_html", "Cannot serve HTML ({:?}) for reason: {:?}", path_converted, err.kind());
            return serve_404()
        }
    };


    Response::builder().
        status(StatusCode::OK).
        body(Body::from(file)).unwrap()
}

