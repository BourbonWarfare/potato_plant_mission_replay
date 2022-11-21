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
use hyper::{Response, Body, StatusCode};
use crate::responses;

pub fn build_json_response(status: StatusCode, json: serde_json::Value) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&json).unwrap()))
    .unwrap()
}

pub fn build_json_response_from_response<T>(response: responses::Response<T>) -> Response<Body>
        where T: serde::Serialize
{
    let (status_code, body_option) = match response {
        responses::Response::Info((status_code, potential_response)) => (status_code, potential_response),
        responses::Response::Success((status_code, potential_response)) => (status_code, potential_response),
        responses::Response::Redirection((status_code, potential_response)) => (status_code, potential_response),
        responses::Response::ClientError((status_code, potential_response)) => (status_code, potential_response),
        responses::Response::ServerError((status_code, potential_response)) => (status_code, potential_response),
    };

    let body = match body_option {
        Some(b) => Body::from(serde_json::to_string(&b).unwrap()),
        None => Body::empty()
    };

    Response::builder()
        .header("Content-Type", "application/json")
        .status(status_code)
        .body(Body::from(body))
    .unwrap()
}
