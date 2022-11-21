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
use std::{
    str::FromStr,
    time::{Duration, Instant},
    collections::HashMap
};
use uuid::Uuid;

use log::{info, warn, error, debug};

/* TODO
 * Generate unique websocket group
 * Assign group to UUID
 * When POSTing a request to create a session, return relevant UUID
 * Connect websocket using UUID as URI path/some other payload method
 * Load stream data
 * Return all when relevant
 */
// A single mission being viewed. Has a UUID and a list of viewers of which we stream to
/// Updates in it's own thread, websockets will read into mission data to figure out next event to
/// send
struct ViewSession {
    current_time: Instant,
}

impl ViewSession {
    fn new() -> ViewSession {
        ViewSession {
            current_time: Instant::now(),
        }
    }
}

struct Lobby {
    view_session: ViewSession,
    unique_id: Uuid,
}

impl Lobby {
    fn new() -> Lobby {
        Lobby {
            view_session: ViewSession::new(),
            unique_id: Uuid::new_v4(),
        }
    }
}

pub struct LobbyHandler {
    lobbies: HashMap<Uuid, Lobby>,
    custom_name_map: HashMap<String, Uuid>
}

impl LobbyHandler {
    pub fn new() -> LobbyHandler {
        LobbyHandler {
            lobbies: HashMap::new(),
            custom_name_map: HashMap::new()
        }
    }

    fn is_uuid_an_active_lobby(&self, lobby_uuid: &Uuid) -> bool {
        return self.lobbies.contains_key(&lobby_uuid)
    }

    pub fn get_lobby_uuid(&self, lobby_id: &str) -> Option<Uuid> {
        match Uuid::from_str(lobby_id) {
            Ok(uuid) => {
                if self.is_uuid_an_active_lobby(&uuid) {
                    Some(uuid)
                } else {
                    None
                }
            },
            Err(_) => {
                if self.custom_name_map.contains_key(lobby_id) {
                    let uuid = self.custom_name_map.get(lobby_id).unwrap().clone();
                    if self.is_uuid_an_active_lobby(&uuid) {
                        return Some(uuid)
                    }
                }
                None
            }
        }
    }

    pub fn create_or_get_lobby_uuid(&mut self, lobby_id: &str) -> Uuid {
        if let Some(uuid) = self.get_lobby_uuid(lobby_id) {
            return uuid
        }

        let new_lobby = Lobby::new();
        let lobby_uuid = new_lobby.unique_id.clone();
        self.custom_name_map.insert(lobby_id.to_string(), lobby_uuid);
        self.lobbies.insert(lobby_uuid, new_lobby);

        lobby_uuid
    }
}


