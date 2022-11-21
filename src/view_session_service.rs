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
    time::{Duration, Instant},
    sync::{Arc, RwLock},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use hyper::service::Service;
use hyper::{Body, Request, Response, Method, body};
use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;
use futures::SinkExt;

use log::{info, warn, debug};

use crate::view_session::LobbyHandler;
use crate::potato_types::Error;
use crate::serve_static::{StaticServer, StaticFile, StaticFileStorage};
use crate::requests;
use crate::responses;
use crate::responses::CanRespond;
use crate::utils;
use crate::json_builder;

pub struct ViewSessionService {
    lobbies: Arc<RwLock<LobbyHandler>>,
    static_server: Arc<StaticServer>
}

impl ViewSessionService {
    fn new(lobbies: Arc<RwLock<LobbyHandler>>, static_server: Arc<StaticServer>) -> ViewSessionService {
        ViewSessionService {
            lobbies: lobbies.clone(),
            static_server
        }
    }

    async fn handle_request(&mut self, mut request: Request<Body>) -> Result<Response<Body>, Error> {
        if hyper_tungstenite::is_upgrade_request(&request) {
            if request.uri().query().is_none() {
                debug!("No query information");
                return Ok(
                    json_builder::build_json_response_from_response(
                        responses::WebSocketFailedConnection::new("No query parameters").build_response(hyper::StatusCode::BAD_REQUEST)    
                    )
                )
            }
            let queries = utils::query_to_hash_map(request.uri());
            let lobby_str = queries.get("lobby-id");
            if let None = lobby_str {
                debug!("Bad query parameters");
                return Ok(
                    json_builder::build_json_response_from_response(
                        responses::WebSocketFailedConnection::new("Bad query parameters").build_response(hyper::StatusCode::BAD_REQUEST)    
                    )
                )
            }

            let lobby_uuid = self.lobbies.read().unwrap().get_lobby_uuid(lobby_str.unwrap());
            if lobby_uuid.is_none() {
                debug!("No lobby exists with provided ID");
                return Ok(
                    json_builder::build_json_response_from_response(
                        responses::WebSocketFailedConnection::new("No lobby exists with provided ID").build_response(hyper::StatusCode::BAD_REQUEST)    
                    )
                )
            }

            let (response, websocket) = hyper_tungstenite::upgrade(&mut request, None)?;

            tokio::spawn(async move {
                /*if let Err(e) = s.serve_websocket(websocket, request).await {
                    warn!(target: "view_session", "Error in websocket connection: {:?}", e);
                }*/
            });

            Ok(response)
        } else {
            self.serve_http(request).await
        }
    }

    async fn serve_websocket(&mut self, websocket: HyperWebsocket, request: Request<Body>) -> Result<(), Error> {
        debug!("New websocket connection");
        let mut websocket = websocket.await?;
        let mut now = Instant::now();
        loop {
            if now.elapsed() > Duration::from_millis(500){
                websocket.send(Message::text("Test")).await?;
                now = Instant::now();
                break;
            }
        }
        Ok(())
    }

    async fn serve_http(&mut self, request: Request<Body>) -> Result<Response<Body>, Error> {
        debug!("New request from path {:?}", request.uri().path());
        let response = match request.method() {
            &Method::GET => self.static_server.serve(request.uri().path()),
            &Method::POST => {
                let uri = request.uri().clone();
                let bytes = body::to_bytes(request.into_body()).await?.to_vec();
                self.handle_http_post(&uri, bytes)
            },
            _ => self.static_server.serve_404()
        };

        Ok(response)
    }

    fn handle_http_post(&mut self, uri: &hyper::Uri, bytes: Vec<u8>) -> Response<Body> {
        let json_parse = serde_json::from_slice(&bytes);
        if let Err(e) = json_parse {
            warn!("Cannot parse request params: {:?}", e);
            return Response::builder().status(hyper::StatusCode::BAD_REQUEST).body(Body::from("")).unwrap();
        }

        let response = match uri.path() {
            "/create_lobby" => {
                let lobby_params: requests::CreateLobby = json_parse.unwrap();
                
                let mut status = responses::LobbyCreated {
                    valid: false,
                    lobby_id: "".to_string()
                };

                if lobby_params.lobby_id.is_ascii() && lobby_params.mission_id.is_ascii() {
                    status.valid = true;

                    let lobby_id_str = &lobby_params.lobby_id[..];
                    status.lobby_id = self.lobbies.write().unwrap().create_or_get_lobby_uuid(lobby_id_str).to_string();
                }

                info!("New lobby request: {:?} || {:?}", lobby_params, status);

                json_builder::build_json_response_from_response(status.build_response(hyper::StatusCode::CREATED))
            }
            _ => {
                Response::builder().status(hyper::StatusCode::NOT_IMPLEMENTED).body(Body::empty()).unwrap()
            }
        };

        response
    }
}

impl Service<Request<Body>> for ViewSessionService {
    type Response = Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut other = ViewSessionService::new(self.lobbies.clone(), self.static_server.clone());
        Box::pin(async move { other.handle_request(req).await })
    }
}

pub struct MakeViewSessionService {
    lobbies: Arc<RwLock<LobbyHandler>>,
    static_server: Arc<StaticServer>
}

impl MakeViewSessionService {
    pub fn new() -> MakeViewSessionService {
        let mut static_server = StaticServer::new("www");
        static_server.register("/", StaticFile::HTML(StaticFileStorage::Disk("index.html")));
        static_server.register("/test.js", StaticFile::JavaScript(StaticFileStorage::Disk("test.js")));

        MakeViewSessionService {
            lobbies: Arc::new(RwLock::new(LobbyHandler::new())),
            static_server: Arc::new(static_server)
        }
    }
}

impl<T> Service<T> for MakeViewSessionService {
    type Response = ViewSessionService;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        debug!("New connection");
        let lobbies = self.lobbies.clone();
        let static_server = self.static_server.clone();
        let fut = async move { Ok(ViewSessionService::new(lobbies, static_server)) };
        Box::pin(fut)
    }}
