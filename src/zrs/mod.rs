use futures::Future;
use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use fsr::*;
use std::sync::Arc;
use std::thread;

use lazy_static::lazy_static;

include!("pb.rs");

pub struct ZrService {
    tx: Arc<broadcast::Sender<Event>>,
}

impl Event {
    pub fn from(e: &fsr::Event) -> Event {
        Event {
            event_id: e.event_id(),
            priority: e.priority(),
            owner: e.owner().to_string(),
            subclass_name: e.subclass_name().to_string(),
            key: e.key(),
            flags: e.flags(),
            headers: e.headers().clone(),
            body: e.body().to_string(),
        }
    }
}

#[tonic::async_trait]
impl zr_server::Zr for ZrService {
    type EventStream = ReceiverStream<Result<EventReply, Status>>;

    async fn event(
        &self,
        request: Request<EventRequest>,
    ) -> Result<Response<Self::EventStream>, Status> {
        fslog!(
            fs::switch_log_level_t::SWITCH_LOG_ERROR,
            "Got a Event request: {:?}",
            request
        );
        let (tx, rx) = mpsc::channel(10);
        let mut sub_rx = self.tx.subscribe();
        tokio::spawn(async move {
            let mut seq = 0u64;
            for v in sub_rx.recv().await.iter() {
                let text = serde_json::to_string(&v).unwrap();
                fslog!(fs::switch_log_level_t::SWITCH_LOG_INFO, "{}", text);
                tx.send(Ok(EventReply {
                    seq,
                    event: Some(v.clone()),
                }))
                .await
                .unwrap();
                seq = seq + 1;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub struct Zrs {
    tx: Arc<broadcast::Sender<Event>>,
    _rx: Arc<broadcast::Receiver<Event>>,
}

impl Zrs {
    #[tokio::main]
    async fn tokio_main<F: Future<Output = ()>>(&self, addr: String, f: F) {
        let addr = addr
            .parse::<std::net::SocketAddr>()
            .expect("Unable to parse grpc socket address");

        let service: ZrService = ZrService {
            tx: self.tx.clone(),
        };

        fslog!(
            fs::switch_log_level_t::SWITCH_LOG_NOTICE,
            "Running zrpc sever on {}",
            addr
        );

        let ret = Server::builder()
            .add_service(zr_server::ZrServer::new(service))
            .serve_with_shutdown(addr, f)
            .await;
        match ret {
            Err(e) => {
                fslog!(
                    fs::switch_log_level_t::SWITCH_LOG_ERROR,
                    "Running zrpc sever: {}",
                    e
                );
            }

            Ok(_) => {
                fslog!(fs::switch_log_level_t::SWITCH_LOG_INFO, "zrpc sever stoped");
            }
        }
    }

    pub fn broadcast(&self, ev: Event) {
        let ret = self.tx.send(ev);
        match ret {
            Err(e) => {
                fslog!(fs::switch_log_level_t::SWITCH_LOG_ERROR, "{}", e);
            }
            _ => {
                fslog!(
                    fs::switch_log_level_t::SWITCH_LOG_DEBUG,
                    "{}",
                    "Event broadcast OK"
                );
            }
        }
    }
}

struct Done {
    sender: Option<broadcast::Sender<u8>>,
}
impl Done {
    pub fn new() -> Done {
        Done { sender: None }
    }

    pub fn set(&mut self, tx: broadcast::Sender<u8>) {
        self.sender = Some(tx);
    }

    pub fn done(&mut self) {
        let _ = self.sender.clone().unwrap().send(1);
    }
}

lazy_static! {
    static ref DONE: Mutex<Done> = Mutex::new(Done::new());
}

pub fn get_instance() -> Arc<Zrs> {
    static mut ZRS: Option<Arc<Zrs>> = None;
    unsafe {
        ZRS.get_or_insert_with(|| {
            let (tx, rx) = broadcast::channel::<Event>(16);
            Arc::new(Zrs {
                tx: Arc::new(tx),
                _rx: Arc::new(rx),
            })
        })
        .clone()
    }
}

pub fn shutdown() {
    DONE.lock().unwrap().done()
}

pub fn broadcast(ev: Event) {
    get_instance().broadcast(ev);
}

pub fn serve(addr: String) {
    let (tx, mut rx) = broadcast::channel::<u8>(1);

    let f = async move {
        let _ = rx.recv();
    };

    DONE.lock().unwrap().set(tx);
    thread::spawn(|| get_instance().tokio_main(addr, f));
}
