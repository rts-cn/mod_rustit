use fsr::*;
use lazy_static::lazy_static;
use md5;
use std::sync::RwLock;
use std::thread;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

include!("pb.rs");
include!("service.rs");

pub struct Zrs {
    _ev_rx: broadcast::Receiver<Event>,
    ev_tx: broadcast::Sender<Event>,
    done: Option<broadcast::Sender<u8>>,
    apply_inbound_acl: Option<String>,
    password: Option<String>,
    threads: i32,
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
            threads: 0,
        }
    }
}

lazy_static! {
    static ref G_ZRS: RwLock<Zrs> = RwLock::new(Zrs::new());
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

impl SystemStatus {
    pub fn from(s: &fsr::SytemStatus) -> SystemStatus {
        SystemStatus {
            uptime: s.uptime,
            version: s.version.clone(),
            ready: s.ready,
            session_total: s.session_total,
            session_active: s.session_active,
            session_peak: s.session_peak,
            session_peak_5min: s.session_peak_5min,
            session_limit: s.session_limit,
            rate_current: s.rate_current,
            rate_max: s.rate_max,
            rate_peak: s.rate_peak,
            rate_peak_5min: s.rate_peak_5min,
            idle_cpu_allowed: s.idle_cpu_allowed,
            idle_cpu_used: s.idle_cpu_used,
            stack_size_current: s.stack_size_current,
            stack_size_max: s.stack_size_max,
        }
    }
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let remote_addr = req.remote_addr();
    if let Some(remote_addr) = remote_addr {
        let remote_addr_str = remote_addr.ip().to_string();
        let apply_inbound_acl: Option<String> = G_ZRS.read().unwrap().apply_inbound_acl.clone();
        if fsr::check_acl(&remote_addr_str, &apply_inbound_acl.unwrap()) {
            return Ok(req);
        }
    }

    let authorization = req.metadata().get("authorization");
    match authorization {
        Some(t) => {
            let password = G_ZRS.read().unwrap().password.clone();
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

    G_ZRS.write().unwrap().password = Some(password);
    G_ZRS.write().unwrap().apply_inbound_acl = Some(acl);

    let (tx, mut rx) = broadcast::channel::<u8>(1);
    let f = async move {
        let _ = rx.recv().await;
    };

    G_ZRS.write().unwrap().done = Some(tx.clone());

    let service = Service {
        tx: G_ZRS.read().unwrap().ev_tx.clone(),
    };

    debug!("Start zrs rpc service {}", addr);
    G_ZRS.write().unwrap().threads += 1;
    let ret = tonic::transport::Server::builder()
        .add_service(zrs_server::ZrsServer::with_interceptor(service, check_auth))
        .serve_with_shutdown(addr, f)
        .await;
    match ret {
        Err(e) => {
            error!("Couldn't start zrpc sever, {}", e);
        }
        Ok(_) => {
            debug!("zrs rpc service thread shutdown.");
        }
    }
    let _ = G_ZRS.write().unwrap().threads -= 1;
}

pub fn broadcast(ev: Event) {
    let ret = G_ZRS.read().unwrap().ev_tx.send(ev);
    match ret {
        Err(e) => {
            error!("{}", e);
        }
        _ => {
            // error!("{}", "Event broadcast OK");
        }
    }
}

pub fn shutdown() {
    let _ = G_ZRS.read().unwrap().done.clone().unwrap().send(1);
    for _ in 0..10 {
        if G_ZRS.read().unwrap().threads <= 0 {
            break;
        }
        thread::sleep(std::time::Duration::from_millis(10));
    }
}

pub fn serve(addr: String, password: String, acl: String) {
    thread::spawn(|| tokio_main(addr, password, acl));
}
