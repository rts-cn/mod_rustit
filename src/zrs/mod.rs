use futures::Future;
use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use fsr::*;
use std::thread;

use lazy_static::lazy_static;

include!("pb.rs");

pub struct ZrService {
    tx: broadcast::Sender<Event>,
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
        let mut remote_addr_str = String::from("");
        let remote_addr = request.remote_addr();

        if let Some(remote_addr) = remote_addr {
            remote_addr_str = remote_addr.to_string();
        }

        info!("Got a subscriber from {}", remote_addr_str);
        let topics = request.into_inner().topics;

        let (tx, rx) = mpsc::channel(10);
        let mut sub_rx = self.tx.subscribe();
        tokio::spawn(async move {
            let mut seq = 0u64;
            loop {
                let v = sub_rx.recv().await;
                match v {
                    Err(e) => {
                        error!("Event broadcast shutdown: {:?}", e);
                        break;
                    }
                    Ok(e) => {
                        let mut pass = false;
                        for topic in &topics {
                            if topic.id == EventTypes::SwitchEventAll as i32 {
                                pass = true;
                                break;
                            } else if (topic.id == EventTypes::SwitchEventCustom as i32)
                                && (topic.subclass == e.subclass_name)
                            {
                                pass = true;
                                break;
                            } else if topic.id == e.event_id as i32 {
                                pass = true;
                                break;
                            }
                        }
                        if pass {
                            let send = tx
                                .send(Ok(EventReply {
                                    seq,
                                    event: Some(e),
                                }))
                                .await;

                            match send {
                                Err(_) => {
                                    notice!("Subscriber disconnect from {}", remote_addr_str);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                };

                debug!("Send Event SEQ: {:?}", seq);
                seq = seq + 1;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// Command sends a single command to the server and returns a response Event.
    async fn command(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandReply>, Status> {
        let req = request.into_inner();
        debug!("{:?}", req);
        let reply = CommandReply { code: 200, message: String::from("OK") };
        Ok(Response::new(reply))
    }
    /// SendMsg sends messages to FreeSWITCH and returns a response Event.
    async fn send_msg(
        &self,
        request: Request<SendMsgRequest>,
    ) -> Result<Response<SendMsgReply>, Status> {
        let req = request.into_inner();
        debug!("{:?}", req);
        let reply = SendMsgReply { code: 200, message: String::from("OK") };
        Ok(Response::new(reply))
    }
}

pub struct Zrs {
    _ev_rx: broadcast::Receiver<Event>,
    ev_tx: broadcast::Sender<Event>,
    done: Option<broadcast::Sender<u8>>,
}

impl Zrs {
    fn new() -> Zrs {
        let (tx, rx) = broadcast::channel::<Event>(16);
        Zrs {
            ev_tx: tx,
            _ev_rx: rx,
            done: None,
        }
    }

    #[tokio::main]
    async fn tokio_main<F: Future<Output = ()>>(
        addr: String,
        event_tx: broadcast::Sender<Event>,
        f: F,
    ) {
        let addr = addr
            .parse::<std::net::SocketAddr>()
            .expect("Unable to parse grpc socket address");

        let service: ZrService = ZrService {
            tx: event_tx.clone(),
        };

        notice!("Running zrpc sever on {}", addr);

        let ret = Server::builder()
            .add_service(zr_server::ZrServer::new(service))
            .serve_with_shutdown(addr, f)
            .await;
        match ret {
            Err(e) => {
                info!("Running zrpc sever: {}", e);
            }

            Ok(_) => {
                warn!("zrpc sever stoped");
            }
        }
    }

    fn broadcast(&self, ev: Event) {
        let ret = self.ev_tx.send(ev);
        match ret {
            Err(e) => {
                error!("{}", e);
            }
            _ => {
                // error!("{}", "Event broadcast OK");
            }
        }
    }

    fn done(&mut self) {
        let _ = self.done.clone().unwrap().send(1);
    }

    fn serve(&mut self, addr: String) {
        let (tx, mut rx) = broadcast::channel::<u8>(1);

        let f = async move {
            let _ = rx.recv().await;
        };

        self.done = Some(tx);

        let ev_sender = self.ev_tx.clone();

        thread::spawn(|| Self::tokio_main(addr, ev_sender, f));
    }
}

lazy_static! {
    static ref G_ZRS: Mutex<Zrs> = Mutex::new(Zrs::new());
}

pub fn shutdown() {
    G_ZRS.lock().unwrap().done()
}

pub fn broadcast(ev: Event) {
    G_ZRS.lock().unwrap().broadcast(ev);
}

pub fn serve(addr: String) {
    G_ZRS.lock().unwrap().serve(addr);
}
