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
    task::{Context, Poll}
};
use hyper::service::Service;
use hyper::{Body, Request, Response, Method, body};
use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

use log::{info, warn, error, debug};

use crate::potato_types::Error;
use crate::serve_static::{StaticServer, StaticFile, StaticFileStorage};
use crate::requests;
use crate::responses;
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
    unique_id: Uuid,
}

impl ViewSession {
    pub fn new() -> ViewSession {
        ViewSession {
            current_time: Instant::now(),
            unique_id: Uuid::new_v4(),
        }
    }

}

pub struct ViewSessionService {
    view_session: Arc<RwLock<ViewSession>>,
    static_server: Arc<StaticServer>
}

impl ViewSessionService {
    fn new(view_session: Arc<RwLock<ViewSession>>, static_server: Arc<StaticServer>) -> ViewSessionService {
        ViewSessionService {
            view_session,
            static_server
        }
    }

    async fn handle_request(&mut self, mut request: Request<Body>) -> Result<Response<Body>, Error> {
        let mut s = ViewSessionService{ view_session: self.view_session.clone(), static_server: self.static_server.clone() };
        if hyper_tungstenite::is_upgrade_request(&request) {
            let (response, websocket) = hyper_tungstenite::upgrade(&mut request, None)?;

            tokio::spawn(async move {
                if let Err(e) = s.serve_websocket(websocket, request).await {
                    warn!(target: "view_session", "Error in websocket connection: {:?}", e);
                }
            });

            Ok(response)
        } else {
            s.serve_http(request).await
        }
    }

    async fn serve_websocket(&mut self, websocket: HyperWebsocket, request: Request<Body>) -> Result<(), Error> {
        debug!(target: "view_session", "New websocket connection");
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
        debug!(target: "view_session", "New request from path {:?}", request.uri().path());
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

    fn build_json_response_from_response<T>(response: responses::Response<T>) -> Response<Body>
        where T: serde::Serialize
    {
        let (status_code, body_option) = match response {
            responses::Response::Info(status_code) => (status_code, None),
            responses::Response::Success((status_code, response_json)) => (
                status_code,
                Some(Body::from(serde_json::to_string(&response_json).unwrap()))
            ),
            responses::Response::Redirection(status_code) => (status_code, None),
            responses::Response::ClientError(status_code) => (status_code, None),
            responses::Response::ServerError(status_code) => (status_code, None)
        };

        let body = match body_option {
            Some(b) => b,
            None => Body::empty()
        };

        Response::builder()
            .header("Content-Type", "application/json")
            .status(status_code)
            .body(Body::from(body))
        .unwrap()
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
                let status = responses::Response::Success((hyper::StatusCode::CREATED, responses::LobbyCreated::new()));
                Self::build_json_response_from_response(status)
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
        let mut other = ViewSessionService::new(self.view_session.clone(), self.static_server.clone());
        Box::pin(async move { other.handle_request(req).await })
    }
}

pub struct MakeViewSessionService {
    session: Arc<RwLock<ViewSession>>,
    static_server: Arc<StaticServer>
}

impl MakeViewSessionService {
    pub fn new() -> MakeViewSessionService {
        let mut static_server = StaticServer::new("www");
        static_server.register("/", StaticFile::HTML(StaticFileStorage::Disk("index.html")));
        static_server.register("/test.js", StaticFile::JavaScript(StaticFileStorage::Disk("test.js")));

        MakeViewSessionService {
            session: Arc::new(RwLock::new(ViewSession::new())),
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
        let session = self.session.clone();
        let static_server = self.static_server.clone();
        let fut = async move { Ok(ViewSessionService::new(session, static_server)) };
        Box::pin(fut)
    }}
