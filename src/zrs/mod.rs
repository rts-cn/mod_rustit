use std::sync::Mutex;
use tokio::sync::broadcast;
use tonic::{Request, Status};
use url::Url;

use fsr::*;
use std::thread;
use lazy_static::lazy_static;

include!("pb.rs");
pub mod event;
pub mod service;
pub mod token;

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
    register_host: Option<String>,
    apply_inbound_acl: Option<String>,
    secret: Option<String>,
}

impl Zrs {
    fn new() -> Zrs {
        let (tx, rx) = broadcast::channel::<Event>(16);
        Zrs {
            ev_tx: tx,
            _ev_rx: rx,
            done: None,
            register_host: None,
            apply_inbound_acl: None,
            secret: None,
        }
    }

    fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
        let remote_addr = req.remote_addr();
        if let Some(remote_addr) = remote_addr {
            let remote_addr_str = remote_addr.ip().to_string();
            let register_host = G_ZRS.lock().unwrap().register_host.clone();
            if remote_addr_str.eq_ignore_ascii_case(&register_host.unwrap()) {
                return Ok(req);
            }
            let apply_inbound_acl = G_ZRS.lock().unwrap().apply_inbound_acl.clone();
            if fsr::check_acl(&remote_addr_str, &apply_inbound_acl.unwrap()) {
                return Ok(req);
            }
        }

        let authorization = req.metadata().get("authorization");
        match authorization {
            Some(t) => {
                let secret_key = G_ZRS.lock().unwrap().secret.clone();
                let token = t.to_str().unwrap();
                let check = token::verify(&secret_key.unwrap(), token);
                match check {
                    Ok(_) => Ok(req),
                    Err(e) => Err(Status::unauthenticated(e)),
                }
            }
            _ => Err(Status::unauthenticated("No valid auth token")),
        }
    }

    #[tokio::main]
    async fn tokio_main(server: Server, node: Info) {
        let addr = server.bind_uri.clone();
        let addr = addr
            .parse::<std::net::SocketAddr>()
            .expect("Unable to parse grpc socket address");

        let url = Url::parse(&server.register_uri);
        match url {
            Err(e) => {
                error!("register uri parse error {}", e);
                return;
            }
            Ok(url) => {
                let host = url.host_str().unwrap();
                G_ZRS.lock().unwrap().register_host = Some(String::from(host));
            }
        }

        let (tx, mut rx) = broadcast::channel::<u8>(1);
        let f = async move {
            let _ = rx.recv().await;
        };

        G_ZRS.lock().unwrap().done = Some(tx.clone());

        let service: service::Service = service::Service {
            tx: G_ZRS.lock().unwrap().ev_tx.clone(),
        };

        notice!("Running zrpc sever on {}", addr);
        tokio::spawn(async move {
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
        });

        let client = zrc_client::ZrcClient::connect(server.register_uri.clone()).await;
        match client {
            Err(e) => {
                error!("Failed to connect to {}:{}", server.register_uri, e);
                return;
            }
            Ok(mut client) => {
                let uuid = node.uuid.clone();
                let request = tonic::Request::new(RegisterRequest { info: Some(node) });
                let response = client.register(request).await;
                match response {
                    Err(e) => {
                        error!("Node registered {}", e);
                    }
                    Ok(response) => {
                        debug!(
                            "Node registered {} [{}]",
                            response.get_ref().message,
                            response.get_ref().code
                        );
                    }
                }
                let mut done = tx.clone().subscribe();
                let _ = done.recv().await;
                let request = tonic::Request::new(UnRegisterRequest { uuid });
                let response = client.un_register(request).await;
                match response {
                    Err(e) => {
                        error!("Node unregistered {}", e);
                    }
                    Ok(response) => {
                        debug!(
                            "Node unregistered {} [{}]",
                            response.get_ref().message,
                            response.get_ref().code
                        );
                    }
                }
            }
        };
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

    fn serve(&mut self, server: Server, node: Info) {
        thread::spawn(|| Self::tokio_main(server, node));
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

pub fn serve(server: Server, node: Info) {
    G_ZRS.lock().unwrap().serve(server, node);
}
