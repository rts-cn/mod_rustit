use fsr::*;
use lazy_static::lazy_static;
use md5;
use std::ffi::CString;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tonic::{Request, Status};

pub mod event;
pub mod pb;
pub mod service;

#[derive(Debug, Clone)]
struct Profile {
    pub event_bind_node: u64,
    pub listen_ip: String,
    pub listen_port: u16,
    pub password: String,
    pub apply_inbound_acl: String,
    pub enable: bool,
}

impl Profile {
    fn new() -> Profile {
        Profile {
            event_bind_node: 0,
            listen_ip: String::from("0.0.0.0"),
            listen_port: 8202,
            password: "".to_string(),
            apply_inbound_acl: "".to_string(),
            enable: false,
        }
    }
}

struct Global {
    running: Mutex<bool>,
    profile: Mutex<Profile>,
    ev_tx: Mutex<Option<broadcast::Sender<pb::Event>>>,
    done_tx: Mutex<Option<mpsc::Sender<u8>>>,
}

impl Global {
    pub fn new() -> Global {
        Global {
            profile: Mutex::new(Profile::new()),
            running: Mutex::new(false),
            ev_tx: Mutex::new(None),
            done_tx: Mutex::new(None),
        }
    }
}

lazy_static! {
    static ref GOLOBAS: Arc<Global> = Arc::new(Global::new());
}

#[tokio::main]
async fn tokio_main(addr: String, password: String, acl: String) {
    let addr = addr.clone();
    let addr = addr
        .parse::<std::net::SocketAddr>()
        .expect("Unable to parse grpc socket address");

    let (tx, mut rx) = broadcast::channel::<pb::Event>(64);
    let (done_tx, mut done_rx) = mpsc::channel(1);

    *GOLOBAS.done_tx.lock().unwrap() = Some(done_tx);
    *GOLOBAS.ev_tx.lock().unwrap() = Some(tx.clone());

    let f = async {
        let _ = done_rx.recv().await;
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

    tokio::spawn(async move {
        loop {
            let ret = rx.recv().await;
            match ret {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                    break;
                }
            }
        }
    });

    let service = service::Service { tx };
    debug!("Start zrs rpc service {}", addr);
    let ret = tonic::transport::Server::builder()
        .add_service(pb::fs_server::FsServer::with_interceptor(
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
}

pub fn shutdown() {
    let id = GOLOBAS.profile.lock().unwrap().event_bind_node;
    if id > 0 {
        fsr::event_unbind(id);
    }

    let tx = GOLOBAS.done_tx.lock().unwrap().clone();
    if tx.is_some() {
        let ret = tx.clone().unwrap().blocking_send(1);
        if let Err(e) = ret {
            error!("{}", e);
        };
    }

    for _ in 1..20 {
        thread::sleep(std::time::Duration::from_millis(200));
        if *GOLOBAS.running.lock().unwrap() == false {
            break;
        }
    }
}

pub fn load_config(cfg: switch_xml_t) {
    unsafe {
        let tmp_str = CString::new("grpc").unwrap();
        let settings_tag = fsr::switch_xml_child(cfg, tmp_str.as_ptr());
        if !settings_tag.is_null() {
            let tmp_str = CString::new("param").unwrap();
            let mut param = fsr::switch_xml_child(settings_tag, tmp_str.as_ptr());
            while !param.is_null() {
                let tmp_str = CString::new("name").unwrap();
                let var = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());
                let tmp_str = CString::new("value").unwrap();
                let val = fsr::switch_xml_attr_soft(param, tmp_str.as_ptr());

                let var = fsr::to_string(var);
                let val = fsr::to_string(val);

                if var.eq_ignore_ascii_case("listen-ip") {
                    GOLOBAS.profile.lock().unwrap().listen_ip = val;
                } else if var.eq_ignore_ascii_case("listen-port") {
                    GOLOBAS.profile.lock().unwrap().listen_port =
                        val.parse::<u16>().unwrap_or(8202);
                } else if var.eq_ignore_ascii_case("password") {
                    GOLOBAS.profile.lock().unwrap().password = val;
                } else if var.eq_ignore_ascii_case("apply-inbound-acl") {
                    GOLOBAS.profile.lock().unwrap().apply_inbound_acl = val;
                } else if var.eq_ignore_ascii_case("enable") {
                    GOLOBAS.profile.lock().unwrap().enable = switch_true(&val);
                }

                param = (*param).next;
            }
        }
    }
}

fn on_event(ev: fsr::Event) {
    let tx = GOLOBAS.ev_tx.lock().unwrap().clone();
    if tx.is_some() {
        let ret = tx.unwrap().send(pb::Event::from(&ev));
        if let Err(e) = ret {
            error!("{}", e);
        }
    }
}

pub fn start(m: &fsr::Module, name: &str) {
    let profile = GOLOBAS.profile.lock().unwrap().clone();
    if profile.enable {
        let bind_uri = format!("{}:{:?}", profile.listen_ip, profile.listen_port);
        let password = profile.password.clone();
        let acl = profile.apply_inbound_acl.clone();
        thread::spawn(move || {
            *GOLOBAS.running.lock().unwrap() = true;
            tokio_main(bind_uri, password, acl);
            *GOLOBAS.running.lock().unwrap() = false;
            debug!("zrs rpc service thread shutdown.");
        });

        thread::sleep(std::time::Duration::from_millis(200));

        let evnode = fsr::event_bind(
            m,
            name,
            switch_event_types_t::SWITCH_EVENT_ALL,
            None,
            on_event,
        );
        GOLOBAS.profile.lock().unwrap().event_bind_node = evnode;
    }
}
