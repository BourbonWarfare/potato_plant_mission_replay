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
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CreateLobby {
    lobby_name: String,
    mission_id: String
}

