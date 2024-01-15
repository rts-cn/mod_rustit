extern crate tonic;

pub mod pb {
    include!("../proto/pb.rs");
}

use futures::lock::Mutex;

use pb::zr_server::{Zr, ZrServer};
use pb::{EventReply, EventRequest};

use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use std::sync::Arc;

pub struct ZrService {
    tx: Arc<Mutex<broadcast::Sender<EventReply>>>,
}

#[tonic::async_trait]
impl Zr for ZrService {
    type EventStream = ReceiverStream<Result<EventReply, Status>>;

    async fn event(
        &self,
        request: Request<EventRequest>,
    ) -> Result<Response<Self::EventStream>, Status> {
        println!("Got a request: {:?}", request);
        let (tx, rx) = mpsc::channel(10);

        let mut sub_rx: broadcast::Receiver<EventReply> = self.tx.lock().await.subscribe();

        tokio::spawn(async move {
            for v in sub_rx.recv().await.iter() {
                tx.send(Ok(v.to_owned())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub struct Zrs {
    tx: broadcast::Sender<EventReply>,
}

impl Zrs {
    #[tokio::main]
    pub async fn listen_and_serve(&self) {
        let addr: std::net::SocketAddr = "[::1]:8888".parse().unwrap();

        let serve: ZrService = ZrService {
            tx: Arc::new(Mutex::new(self.tx.clone())),
        };

        let r = Server::builder().add_service(ZrServer::new(serve));

        tokio::spawn(async move {
            let _ = r.serve(addr).await;
        });
    }

    pub async fn broadcast(&self, ev: String) {
        let _ = self.tx.send(EventReply { event: ev });
    }
}

pub fn get_instance() -> Arc<Zrs> {
    static mut ZRS: Option<Arc<Zrs>> = None;
    unsafe {
        ZRS.get_or_insert_with(|| {
            let (tx, _) = broadcast::channel::<EventReply>(16);
            Arc::new(Zrs { tx })
        })
        .clone()
    }
}
