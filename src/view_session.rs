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
use hyper::{Body, Request, Response, StatusCode, Method, header::{HeaderValue, HeaderName}};
use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

use log::{info, warn, error, debug};

use crate::potato_types::Error;
use crate::serve_static::{StaticServer, StaticFile, StaticFileStorage};

/// A single mission being viewed. Has a UUID and a list of viewers of which we stream to
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

    fn handle_request(&mut self, mut request: Request<Body>) -> Result<Response<Body>, Error> {
        if hyper_tungstenite::is_upgrade_request(&request) {
            let (response, websocket) = hyper_tungstenite::upgrade(&mut request, None)?;

            let mut s = ViewSessionService{ view_session: self.view_session.clone(), static_server: self.static_server.clone() };
            tokio::spawn(async move {
                if let Err(e) = s.serve_websocket(websocket, request).await {
                    warn!(target: "view_session", "Error in websocket connection: {:?}", e);
                }
            });

            Ok(response)
        } else {
            self.serve_http(request)
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

    /* TODO
     * Generate unique websocket group
     * Assign group to UUID
     * When POSTing a request to create a session, return relevant UUID
     * Connect websocket using UUID as URI path/some other payload method
     * Load stream data
     * Return all when relevant
     */
    fn serve_http(&mut self, mut request: Request<Body>) -> Result<Response<Body>, Error> {
        debug!(target: "view_session", "New request from path {:?}", request.uri().path());
        let response = match (request.method(), request.uri().path()) {
            (&Method::GET, path) => self.static_server.serve(path),
            (&Method::PUT, "/create_lobby") => {
                debug!(target: "view_session", "Request payload: {:?}", request.body());
                Response::builder()
                    .header("Content-Type", "application/json")
                    .body(Body::from("{ \"test\": 500 }"))
                .unwrap()
            },
            _ => self.static_server.serve_404()
        };

        Ok(response)
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
        let res = self.handle_request(req);
        Box::pin(async { res })
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
