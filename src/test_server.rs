use hyper::service::Service;
use hyper::{Body, Request, Response, Server};

use hyper_tungstenite::{tungstenite, HyperWebsocket};
use std::time::{Instant, Duration};
use futures::{sink::SinkExt, stream::StreamExt};
use tungstenite::Message;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::{Arc, RwLock, atomic::AtomicU32, atomic::Ordering};

type Counter = AtomicU32;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct TestLooper {
    last_update: Instant,
    counter: Arc<Counter>
}

impl TestLooper {
    fn new() -> TestLooper {
        TestLooper{
            last_update: Instant::now(),
            counter: Arc::new(AtomicU32::new(0))
        }
    }
}

pub struct Svc {
    looper: Arc<RwLock<TestLooper>>
}

impl Svc {
    fn handle_request(&mut self, mut request: Request<Body>) -> Result<Response<Body>, Error> {
        if hyper_tungstenite::is_upgrade_request(&request) {
            let (response, websocket) = hyper_tungstenite::upgrade(&mut request, None)?;

            let mut s = Svc{looper: self.looper.clone()};
            tokio::spawn(async move {
                if let Err(e) = s.serve_websocket(websocket).await {
                    eprintln!("Error in websocket connection: {}", e);
                }
            });

            Ok(response)
        } else {
            Ok(Response::new(Body::from("Hello HTTP!")))
        }
    }

    async fn serve_websocket(&mut self, websocket: HyperWebsocket) -> Result<(), Error> {
        let mut websocket = websocket.await?;
        let mut now = Instant::now();
        loop {
            if now.elapsed() > Duration::from_millis(500){
                websocket.send(Message::text(format!("counter: {:?}", self.looper.read().unwrap().counter))).await?;
                now = Instant::now()
            }
        }
        Ok(())
    }
}

impl Service<Request<Body>> for Svc {
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

pub struct MakeSvc {
    looper: Arc<RwLock<TestLooper>>
}

impl MakeSvc {
    pub fn new() -> MakeSvc {
        let looper = Arc::new(RwLock::new(TestLooper::new()));
        let moved_looper = looper.clone();
        tokio::spawn(async move {
            loop {
                moved_looper.write().unwrap().counter.fetch_add(1, Ordering::SeqCst);
                if moved_looper.read().unwrap().last_update.elapsed() > Duration::from_secs(1) {
                    moved_looper.write().unwrap().counter.swap(0, Ordering::Relaxed);
                    moved_looper.write().unwrap().last_update = Instant::now();
                }
            }
        });

        MakeSvc {
            looper: looper.clone()
        }
    }
}

impl<T> Service<T> for MakeSvc {
    type Response = Svc;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        println!("conn");
        let looper = self.looper.clone();
        let fut = async move { Ok(Svc {looper: looper}) };
        Box::pin(fut)
    }
}
