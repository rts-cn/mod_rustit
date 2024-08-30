use axum::{routing::get, Router};
use lazy_static::lazy_static;
use std::ffi::CString;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use switch_sys::*;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

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
            listen_port: 8203,
            password: "".to_string(),
            apply_inbound_acl: "".to_string(),
            enable: false,
        }
    }
}

struct Global {
    running: Mutex<bool>,
    profile: Mutex<Profile>,
    ev_tx: Mutex<Option<broadcast::Sender<String>>>,
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

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

pub fn shutdown() {
    let id = GOLOBAS.profile.lock().unwrap().event_bind_node;
    if id > 0 {
        switch_sys::event_unbind(id);
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
            debug!("jsapi server shutdown");
            *GOLOBAS.done_tx.lock().unwrap() = None;
            *GOLOBAS.ev_tx.lock().unwrap() = None;
            break;
        }
    }
}

pub fn load_config(cfg: switch_xml_t) {
    unsafe {
        let tmp_str = CString::new("jsapi").unwrap();
        let settings_tag = switch_sys::switch_xml_child(cfg, tmp_str.as_ptr());
        if !settings_tag.is_null() {
            let tmp_str = CString::new("param").unwrap();
            let mut param = switch_sys::switch_xml_child(settings_tag, tmp_str.as_ptr());
            while !param.is_null() {
                let tmp_str = CString::new("name").unwrap();
                let var = switch_sys::switch_xml_attr_soft(param, tmp_str.as_ptr());
                let tmp_str = CString::new("value").unwrap();
                let val = switch_sys::switch_xml_attr_soft(param, tmp_str.as_ptr());

                let var = switch_sys::switch_to_string(var);
                let val = switch_sys::switch_to_string(val);

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

fn on_event(ev: switch_sys::Event) {
    let tx = GOLOBAS.ev_tx.lock().unwrap().clone();
    if tx.is_some() {
        let ret = tx.unwrap().send(ev.string());
        if let Err(e) = ret {
            error!("{}", e);
        }
    }
}

#[tokio::main]
async fn tokio_main(address: String) {
    let (tx, mut rx) = broadcast::channel::<String>(64);
    let (done_tx, mut done_rx) = mpsc::channel(1);

    *GOLOBAS.done_tx.lock().unwrap() = Some(done_tx);
    *GOLOBAS.ev_tx.lock().unwrap() = Some(tx.clone());

    let f = async move {
        let _ = done_rx.recv().await;
        debug!("jsapi server shutdown");
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
    debug!("Start jsapi service {}", address);
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(f)
        .await
        .unwrap();
}

pub fn start(m: &switch_sys::Module, name: &str) {
    let profile = GOLOBAS.profile.lock().unwrap().clone();
    if profile.enable {
        let bind_uri = format!("{}:{:?}", profile.listen_ip, profile.listen_port);
        thread::spawn(move || {
            *GOLOBAS.running.lock().unwrap() = true;
            tokio_main(bind_uri);
            *GOLOBAS.running.lock().unwrap() = false;
            debug!("jsapi service thread shutdown.");
        });
        thread::sleep(std::time::Duration::from_millis(200));
        let evnode = switch_sys::event_bind(
            m,
            name,
            switch_event_types_t::SWITCH_EVENT_ALL,
            None,
            on_event,
        );
        GOLOBAS.profile.lock().unwrap().event_bind_node = evnode;
    }
}
