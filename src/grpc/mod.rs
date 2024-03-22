use fsr::*;
use lazy_static::lazy_static;
use md5;
use std::sync::RwLock;
use std::thread;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tonic::{Request, Status};

pub mod event;
pub mod pb;
pub mod service;

pub struct Zrs {
    ev_tx: broadcast::Sender<pb::Event>,
    done: mpsc::Sender<u8>,
}

struct Global {
    running: bool,
    zrs: Option<Zrs>,
}
impl Global {
    pub fn new() -> Global {
        Global {
            running: false,
            zrs: None,
        }
    }
}

lazy_static! {
    static ref GOLOBAS: RwLock<Global> = RwLock::new(Global::new());
}

fn tokio_main(
    addr: String,
    password: String,
    acl: String,
    tx: broadcast::Sender<pb::Event>,
    mut rx: broadcast::Receiver<pb::Event>,
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
            if fsr::check_acl(&remote_addr_str, &acl) {
                return Ok(req);
            }
        }

        let authorization = req.metadata().get("authorization");
        match authorization {
            Some(t) => {
                let digest = format!("bearer {:x}", md5::compute(&password));
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
        let service = service::Service { tx };
        debug!("Start zrs rpc service {}", addr);
        let ret = tonic::transport::Server::builder()
            .add_service(pb::switch_server::SwitchServer::with_interceptor(
                service, check_auth,
            ))
            .serve_with_shutdown(addr, f)
            .await;
        match ret {
            Err(e) => {
                error!("Couldn't start zrpc sever, {}", e);
            }
            Ok(_) => {}
        }
    });
}

pub fn broadcast(ev: fsr::Event) {
    let zrs = &GOLOBAS.read().unwrap().zrs;
    if let Some(zrs) = zrs {
        let ret = zrs.ev_tx.send(pb::Event::from(&ev));
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

    for _ in 1..20 {
        thread::sleep(std::time::Duration::from_millis(200));
        if GOLOBAS.read().unwrap().running == false {
            break;
        }
    }
    GOLOBAS.write().unwrap().zrs = None;
}

pub fn serve(addr: String, password: String, acl: String) {
    lazy_static::initialize(&GOLOBAS);
    let (done_tx, done_rx) = mpsc::channel(1);
    let (tx, rx) = broadcast::channel::<pb::Event>(64);
    let zrs = Zrs {
        ev_tx: tx.clone(),
        done: done_tx,
    };
    GOLOBAS.write().unwrap().zrs = Some(zrs);
    thread::spawn(move || {
        GOLOBAS.write().unwrap().running = true;
        tokio_main(addr, password, acl, tx, rx, done_rx);
        GOLOBAS.write().unwrap().running = false;
        debug!("zrs rpc service thread shutdown.");
    });
}