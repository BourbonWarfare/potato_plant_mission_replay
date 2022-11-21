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
use serde::Serialize;
use hyper::StatusCode;

pub enum Response<T> {
    Info((StatusCode, Option<T>)),
    Success((StatusCode, Option<T>)),
    Redirection((StatusCode, Option<T>)),
    ClientError((StatusCode, Option<T>)),
    ServerError((StatusCode, Option<T>))
}

pub trait CanRespond 
    where Self: Sized
{
    fn build_response(self, status_code: StatusCode) -> Response<Self> {
        if status_code.is_informational() {
            return Response::Info((status_code, Some(self)))
        } else if status_code.is_success() {
            return Response::Success((status_code, Some(self)))
        } else if status_code.is_redirection() {
            return Response::Redirection((status_code, Some(self)))
        } else if status_code.is_client_error() {
            return Response::ClientError((status_code, Some(self)))
        }
        Response::ServerError((status_code, Some(self)))
    }
}

#[derive(Serialize, Debug)]
pub struct LobbyCreated {
    pub valid: bool,
    pub lobby_id: String
}
impl CanRespond for LobbyCreated {}

#[derive(Serialize, Debug)]
pub struct WebSocketFailedConnection {
    pub valid: bool,
    pub message: String
}

impl WebSocketFailedConnection {
    pub fn new(msg: &str) -> WebSocketFailedConnection {
        WebSocketFailedConnection {
            valid: false,
            message: msg.to_string()
        }
    }
}
impl CanRespond for WebSocketFailedConnection {}

