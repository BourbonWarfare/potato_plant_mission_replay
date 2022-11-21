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
mod view_session;
mod view_session_service;
mod json_builder;
mod potato_types;
mod serve_static;
mod requests;
mod responses;
mod utils;

use crate::potato_types::Error;
use pretty_env_logger;
use std::env;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env::set_var("RUST_APP_LOG", "trace");
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    let addr: std::net::SocketAddr = "[::1]:3000".parse()?;
    info!(target: "potato_plant_replay", "Listening on {:?}", addr);
    let svc = view_session_service::MakeViewSessionService::new();
    let server = hyper::Server::bind(&addr).serve(svc);

    server.await?;

    env::remove_var("RUST_APP_LOG");
    Ok(())
}
