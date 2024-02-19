use fsr::*;
use lazy_static::lazy_static;
use md5;
use std::sync::Mutex;
use std::thread;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

include!("pb.rs");
include!("service.rs");

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

pub struct Server {
    pub bind_uri: String,
    pub register_uri: String,
    pub secret: String,
    pub apply_inbound_acl: String,
}

pub struct Zrs {
    _ev_rx: broadcast::Receiver<Event>,
    ev_tx: broadcast::Sender<Event>,
    done: Option<broadcast::Sender<u8>>,
    apply_inbound_acl: Option<String>,
    password: Option<String>,
}

impl Zrs {
    fn new() -> Zrs {
        let (tx, rx) = broadcast::channel::<Event>(16);
        Zrs {
            ev_tx: tx,
            _ev_rx: rx,
            done: None,
            apply_inbound_acl: None,
            password: None,
        }
    }

    fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
        let remote_addr = req.remote_addr();
        if let Some(remote_addr) = remote_addr {
            let remote_addr_str = remote_addr.ip().to_string();
            let apply_inbound_acl = G_ZRS.lock().unwrap().apply_inbound_acl.clone();
            if fsr::check_acl(&remote_addr_str, &apply_inbound_acl.unwrap()) {
                return Ok(req);
            }
        }

        let authorization = req.metadata().get("authorization");
        match authorization {
            Some(t) => {
                let password = G_ZRS.lock().unwrap().password.clone();
                let password = password.unwrap();
                let digest = format!("bearer {:x}", md5::compute(password));
                let token = t.to_str().unwrap();
                if digest.eq_ignore_ascii_case(token) {
                    Ok(req)
                } else {
                    Err(Status::unauthenticated(
                        "authentication failure wrong password",
                    ))
                }
            }
            _ => Err(Status::unauthenticated("No valid auth token")),
        }
    }

    #[tokio::main]
    async fn tokio_main(addr: String, password: String, acl: String) {
        let addr = addr.clone();
        let addr = addr
            .parse::<std::net::SocketAddr>()
            .expect("Unable to parse grpc socket address");

        G_ZRS.lock().unwrap().password = Some(password);
        G_ZRS.lock().unwrap().apply_inbound_acl = Some(acl);

        let (tx, mut rx) = broadcast::channel::<u8>(1);
        let f = async move {
            let _ = rx.recv().await;
        };

        G_ZRS.lock().unwrap().done = Some(tx.clone());

        let service = Service {
            tx: G_ZRS.lock().unwrap().ev_tx.clone(),
        };

        notice!("Running zrpc sever on {}", addr);
        let ret = tonic::transport::Server::builder()
            .add_service(zrs_server::ZrsServer::with_interceptor(
                service,
                Self::check_auth,
            ))
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

    fn serve(&mut self, addr: String, password: String, acl: String) {
        thread::spawn(|| Self::tokio_main(addr, password, acl));
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

pub fn serve(addr: String, password: String, acl: String) {
    G_ZRS.lock().unwrap().serve(addr, password, acl);
}
