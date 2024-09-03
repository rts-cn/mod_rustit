use axum::http::StatusCode;
use axum::response::sse::Event;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum::Router;
use futures::stream::Stream;
use futures::StreamExt;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
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
    ev_tx: Mutex<Option<broadcast::Sender<switch_sys::Event>>>,
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
            debug!("HTTP API server shutdown");
            *GOLOBAS.done_tx.lock().unwrap() = None;
            *GOLOBAS.ev_tx.lock().unwrap() = None;
            break;
        }
    }
}

pub fn load_config(cfg: switch_xml_t) {
    unsafe {
        let tmp_str = CString::new("api").unwrap();
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
        let ret = tx.unwrap().send(ev);
        if let Err(e) = ret {
            error!("{}", e);
        }
    }
}

#[derive(Serialize)]
struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {}

#[derive(Debug, Serialize, Deserialize)]
struct ModuleRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JSAPIRequest {
    command: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendEventRequest {
    event_id: u32,
    subclass_name: String,
    headers: std::collections::HashMap<String, String>,
    body: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendMsgRequest {
    uuid: String,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandRequest {
    command: String,
    args: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Topic {
    name: String,
    subclass_name: String,
    id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscribeRequest {
    topics: Vec<Topic>,
}

async fn auth_middleware(
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let remote_addr_str = addr.ip().to_string();
    let acl = GOLOBAS.profile.lock().unwrap().apply_inbound_acl.clone();
    if switch_sys::check_acl(&remote_addr_str, &acl) {
        debug!(
            "[{}] {} from {}",
            request.method(),
            request.uri(),
            addr.to_string()
        );
        return next.run(request).await;
    }
    debug!(
        "API [{}] {} from {} authentication failure",
        request.method(),
        request.uri(),
        remote_addr_str
    );
    ApiResponse {
        code: 401,
        message: StatusCode::UNAUTHORIZED.to_string(),
        data: Some(()),
    }
    .into_response()
}

// Reload XML
async fn hander_reload_xml() -> impl IntoResponse {
    let handle = tokio::task::spawn_blocking(|| switch_sys::api_exec("reloadxml", ""));
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: Some(()),
        },
        Ok(msg) => ApiResponse {
            code: 200,
            message: msg,
            data: Some(()),
        },
    }
}

// Reload ACL
async fn hander_reload_acl() -> impl IntoResponse {
    let handle = tokio::task::spawn_blocking(|| switch_sys::api_exec("reloadacl", ""));
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: Some(()),
        },
        Ok(msg) => ApiResponse {
            code: 200,
            message: msg,
            data: Some(()),
        },
    }
}

/// Reload mod
async fn hander_reload_mod(Json(request): Json<ModuleRequest>) -> impl IntoResponse {
    let mut cmd = "reload";
    let mut args = request.name;

    if args.contains("mod_rustit") {
        cmd = "bgapi";
        args = String::from("reload mod_rustit");
    }

    let handle = tokio::task::spawn_blocking(move || switch_sys::api_exec(cmd, &args));
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: Some(()),
        },
        Ok(msg) => ApiResponse {
            code: 200,
            message: msg,
            data: Some(()),
        },
    }
}

/// Load mod
async fn hander_load_mod(Json(request): Json<ModuleRequest>) -> impl IntoResponse {
    let handle = tokio::task::spawn_blocking(move || switch_sys::api_exec("load", &request.name));
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: Some(()),
        },
        Ok(msg) => ApiResponse {
            code: 200,
            message: msg,
            data: Some(()),
        },
    }
}

/// Unload mod
async fn hander_unload_mod(Json(request): Json<ModuleRequest>) -> impl IntoResponse {
    if request.name.contains("mod_rustit") {
        return ApiResponse {
            code: 501,
            message: String::from("-ERR Module mod_rustit is in use, cannot unload"),
            data: Some(()),
        };
    }

    let handle = tokio::task::spawn_blocking(move || switch_sys::api_exec("unload", &request.name));
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: Some(()),
        },
        Ok(msg) => ApiResponse {
            code: 200,
            message: msg,
            data: Some(()),
        },
    }
}

/// JSAPI
async fn hander_jsapi(Json(request): Json<JSAPIRequest>) -> impl IntoResponse {
    let cmd = serde_json::json!({"data": request.args, "command": &request.command});
    let cmd = cmd.to_string();
    let mut json_format = false;
    if request.command.eq_ignore_ascii_case("fsapi") && cmd.find("json").is_some() {
        json_format = true;
    }
    let handle = tokio::task::spawn_blocking(move || switch_sys::json_api_exec(&cmd));
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: None,
        },
        Ok(message) => {
            let json_value: Result<serde_json::Value, serde_json::Error> =
                serde_json::from_str(&message);
            if let Ok(mut json_value) = json_value {
                if json_format {
                    let object = json_value.as_object_mut();
                    if let Some(object) = object {
                        let response = object.get_mut("response");
                        if let Some(response) = response {
                            let response = response.as_object_mut();
                            if let Some(response) = response {
                                let message = response.get("message");
                                if let Some(message) = message {
                                    let msg = message.as_str().unwrap_or("{}");
                                    let msg_value: Result<serde_json::Value, serde_json::Error> =
                                        serde_json::from_str(msg);
                                    if let Ok(msg_value) = msg_value {
                                        object.remove("response");
                                        object.insert("response".to_string(), msg_value);
                                    }
                                }
                            }
                        }
                    }
                }
                return ApiResponse {
                    code: 200,
                    message: "OK".to_string(),
                    data: Some(json_value),
                };
            } else {
                return ApiResponse {
                    code: 200,
                    message,
                    data: None,
                };
            }
        }
    }
}

/// SendMsg sends messages to FreeSWITCH and returns a response..
async fn hander_send_msg(Json(request): Json<SendMsgRequest>) -> impl IntoResponse {
    let handle: tokio::task::JoinHandle<Result<String, String>> =
        tokio::task::spawn_blocking(move || switch_sys::sendmsg(&request.uuid, request.headers));
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: Some(()),
        },
        Ok(msg) => ApiResponse {
            code: 200,
            message: msg,
            data: Some(()),
        },
    }
}

/// SendEvent sends event to FreeSWITCH.
async fn hander_send_event(Json(request): Json<SendEventRequest>) -> impl IntoResponse {
    let handle = tokio::task::spawn_blocking(move || {
        switch_sys::sendevent(
            request.event_id,
            &request.subclass_name,
            request.headers,
            &request.body,
        )
    });
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: Some(()),
        },
        Ok(msg) => ApiResponse {
            code: 200,
            message: msg,
            data: Some(()),
        },
    }
}

/// Command sends a single command to the server and returns a response Event.
async fn hander_command(Json(request): Json<CommandRequest>) -> impl IntoResponse {
    let mut cmd = request.command;
    let mut args = request.args;

    let mut json_format = false;
    if cmd.find("json").is_some() || args.find("json").is_some() {
        json_format = true;
    }

    if cmd.contains("reload") && args.contains("mod_rustit") {
        cmd = String::from("bgapi");
        args = String::from("reload mod_rustit");
    } else if cmd.contains("unload") && args.contains("mod_rustit") {
        return ApiResponse {
            code: 501,
            message: String::from("Module mod_rustit is in use, cannot unload"),
            data: None,
        };
    }

    let handle = tokio::task::spawn_blocking(move || switch_sys::api_exec(&cmd, &args));
    let res = handle.await.unwrap();
    match res {
        Err(e) => ApiResponse {
            code: 500,
            message: e,
            data: None,
        },
        Ok(msg) => {
            if json_format {
                let json_value: Result<serde_json::Value, serde_json::Error> =
                    serde_json::from_str(&msg);
                if let Ok(json_value) = json_value {
                    let value = serde_json::json!({"response": json_value, "status": "success"});
                    return ApiResponse {
                        code: 200,
                        message: "OK".to_string(),
                        data: Some(value),
                    };
                }
            }
            return ApiResponse {
                code: 200,
                message: msg,
                data: None,
            };
        }
    }
}

async fn hander_event(
    Json(request): Json<SubscribeRequest>,
) -> axum::response::Sse<impl Stream<Item = Result<axum::response::sse::Event, String>>> {
    let tx = GOLOBAS.ev_tx.lock().unwrap().clone().unwrap();
    let rx = tx.subscribe();
    let mut topics: Vec<Topic> = Vec::new();
    for topic in request.topics {
        let name = topic.name.to_ascii_uppercase();
        let id = switch_sys::switch_event_types_t::from_str(&name);
        if let Some(id) = id {
            topics.push(Topic {
                id: Some(id.0),
                subclass_name: topic.subclass_name,
                name,
            });
        }
    }
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(move |result| {
        result
            .map(|e: switch_sys::Event| {
                let mut pass = false;
                for topic in &topics {
                    match topic.id {
                        Some(event_id) => {
                            if event_id == switch_sys::switch_event_types_t::SWITCH_EVENT_ALL.0 {
                                pass = true;
                                break;
                            } else if (event_id
                                == switch_sys::switch_event_types_t::SWITCH_EVENT_CUSTOM.0)
                                && (topic.subclass_name == e.subclass_name)
                            {
                                pass = true;
                                break;
                            } else if event_id == e.event_id {
                                pass = true;
                                break;
                            }
                        }
                        None => {
                            continue;
                        }
                    }
                }
                if pass {
                    Event::default().data(e.json())
                } else {
                    Event::default().data("")
                }
            })
            .map_err(|_| "".to_string())
    });
    axum::response::Sse::new(stream)
}

#[tokio::main]
async fn tokio_main(address: String) {
    let (tx, mut rx) = broadcast::channel::<switch_sys::Event>(64);
    let (done_tx, mut done_rx) = mpsc::channel(1);

    *GOLOBAS.done_tx.lock().unwrap() = Some(done_tx);
    *GOLOBAS.ev_tx.lock().unwrap() = Some(tx.clone());

    let f = async move {
        let _ = done_rx.recv().await;
        debug!("HTTP API server done");
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
    debug!("Start HTTP API service {}", address);
    // build our application with a route
    let app = Router::new()
        .route("/api/reloadxml", post(hander_reload_xml))
        .route("/api/reloadacl", post(hander_reload_acl))
        .route("/api/module/reload", post(hander_reload_mod))
        .route("/api/module/load", post(hander_load_mod))
        .route("/api/module/unload", post(hander_unload_mod))
        .route("/api/jsapi", post(hander_jsapi))
        .route("/api/send/event", post(hander_send_event))
        .route("/api/send/msg", post(hander_send_msg))
        .route("/api/command", post(hander_command))
        .route("/api/sse/event", get(hander_event))
        .route_layer(axum::middleware::from_fn(auth_middleware));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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
            debug!("HTTP API service thread shutdown.");
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
