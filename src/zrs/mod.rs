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
    ev_tx: broadcast::Sender<Event>,
    done: mpsc::Sender<u8>,
}

struct Global {
    zrs: Option<Zrs>,
}
impl Global {
    pub fn new() -> Global {
        Global { zrs: None }
    }
}

lazy_static! {
    static ref GOLOBAS: RwLock<Global> = RwLock::new(Global::new());
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

fn tokio_main(
    addr: String,
    password: String,
    acl: String,
    tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
    mut done: mpsc::Receiver<u8>,
) {
    let addr = addr.clone();
    let addr = addr
        .parse::<std::net::SocketAddr>()
        .expect("Unable to parse grpc socket address");

    let f = async {
        let _ = done.recv().await;
        done.close();
        drop(done);
    };

    let check_auth = move |req: Request<()>| -> Result<Request<()>, Status> {
        let remote_addr = req.remote_addr();
        if let Some(remote_addr) = remote_addr {
            let remote_addr_str = remote_addr.ip().to_string();
            let apply_inbound_acl = acl.clone();
            if fsr::check_acl(&remote_addr_str, &apply_inbound_acl) {
                return Ok(req);
            }
        }

        let authorization = req.metadata().get("authorization");
        match authorization {
            Some(t) => {
                let password = password.clone();
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
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(async move {
        loop {
            let ret = rx.recv().await;
            match ret {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
        }
        drop(rx);
    });

    rt.block_on(async {
        let service = Service { tx };
        debug!("Start zrs rpc service {}", addr);
        let ret = tonic::transport::Server::builder()
            .add_service(zrs_server::ZrsServer::with_interceptor(service, check_auth))
            .serve_with_shutdown(addr, f)
            .await;
        match ret {
            Err(e) => {
                error!("Couldn't start zrpc sever, {}", e);
            }
            Ok(_) => {}
        }
    });

    rt.shutdown_timeout(tokio::time::Duration::from_millis(100));
    debug!("zrs rpc service thread shutdown.");
}

pub fn broadcast(ev: Event) {
    let zrs = &GOLOBAS.read().unwrap().zrs;
    if let Some(zrs) = zrs {
        let ret = zrs.ev_tx.send(ev);
        if let Err(e) = ret {
            error!("{}", e);
        }
    }
}

pub fn shutdown() {
    let r = GOLOBAS.read().unwrap();
    let zrs = &r.zrs;
    if let Some(zrs) = zrs {
        let ret = zrs.done.blocking_send(1);
        if let Err(e) = ret {
            error!("{}", e);
        };
    }
    drop(r);
    GOLOBAS.write().unwrap().zrs = None;
}

pub fn serve(addr: String, password: String, acl: String) {
    lazy_static::initialize(&GOLOBAS);
    let (done_tx, done_rx) = mpsc::channel(1);
    let (tx, rx) = broadcast::channel::<Event>(64);
    let zrs = Zrs {
        ev_tx: tx.clone(),
        done: done_tx,
    };
    GOLOBAS.write().unwrap().zrs = Some(zrs);
    thread::spawn(move || {
        tokio_main(addr, password, acl, tx, rx, done_rx);
    });
}
