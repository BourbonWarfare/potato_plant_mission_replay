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
use hyper::{Body, Request, Response, StatusCode};
use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

use crate::potato_types::Error;

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

struct ViewSessionService {
    view_session: Arc<RwLock<ViewSession>>    
}

impl ViewSessionService {
    fn handle_request(&mut self, mut request: Request<Body>) -> Result<Response<Body>, Error> {
        if hyper_tungstenite::is_upgrade_request(&request) {
            let (response, websocket) = hyper_tungstenite::upgrade(&mut request, None)?;

            let mut s = ViewSessionService{ view_session: self.view_session.clone() };
            tokio::spawn(async move {
                if let Err(e) = s.serve_websocket(websocket).await {
                    eprintln!("Error in websocket connection: {}", e);
                }
            });

            Ok(response)
        } else {
            self.serve_http(request)
        }
    }

    async fn serve_websocket(&mut self, websocket: HyperWebsocket) -> Result<(), Error> {
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
     * Load static website data
     * Return all when relevant
     */

    fn serve_http(&mut self, mut request: Request<Body>) -> Result<Response<Body>, Error> {
        let mut response = Response::new(Body::empty());
        match (request.method(), request.uri().path()) {
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        }

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
    session: Arc<RwLock<ViewSession>>
}

impl MakeViewSessionService {
    pub fn new() -> MakeViewSessionService {
        MakeViewSessionService {
            session: Arc::new(RwLock::new(ViewSession::new()))
        }
    }
}

impl<T> Service<T> for MakeViewSessionService {
    type Response = ViewSession;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        println!("conn");
        let fut = async move { Ok(ViewSession::new()) };
        Box::pin(fut)
    }}
