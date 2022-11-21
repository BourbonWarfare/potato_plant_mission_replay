/*
    potato_plant_replay - controls replaying a given mission. clients connect to a websocket and recieve data
    copyright (c) 2022  bailey danyluk

    this program is free software: you can redistribute it and/or modify
    it under the terms of the gnu general public license as published by
    the free software foundation, either version 3 of the license, or
    (at your option) any later version.

    this program is distributed in the hope that it will be useful,
    but without any warranty; without even the implied warranty of
    merchantability or fitness for a particular purpose.  see the
    gnu general public license for more details.

    you should have received a copy of the gnu general public license
    along with this program.  if not, see <https://www.gnu.org/licenses/>.
*/
use std::collections::HashMap;
use hyper::{Uri, http::uri::PathAndQuery};

pub fn query_to_hash_map(uri: &Uri) -> HashMap<&str, &str> {
    if uri.query().is_none() {
        return HashMap::new()
    }

    let mut return_map = HashMap::new();

    uri.query().unwrap()
        .split('&')
        .map(|query| query.split('=').collect())
        .for_each(|query_vec: Vec<&str>| {
            if query_vec.len() == 2 {
                return_map.insert(query_vec[0], query_vec[1]);
            } else if query_vec.len() == 1 {
                return_map.insert(query_vec[0], "");
            }
        });

    return_map
}

