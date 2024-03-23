use fsr::*;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use super::pb::*;

pub struct Service {
    pub tx: broadcast::Sender<super::pb::Event>,
}

struct Topics {
    id: u32,
    subclass_name: String,
}

#[tonic::async_trait]
impl super::pb::fs_server::Fs for Service {
    type SubscribeStream = ReceiverStream<Result<super::pb::Event, Status>>;

    /// Event Stream
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let mut remote_addr_str = String::from("");
        let remote_addr = request.remote_addr();

        if let Some(remote_addr) = remote_addr {
            remote_addr_str = remote_addr.to_string();
        }

        info!("Got a subscriber from {}", remote_addr_str);
        let mut topics: Vec<Topics> = Vec::new();
        for topic in request.into_inner().topics {
            let name = topic.event_name.to_ascii_uppercase();
            let id = super::event::EventTypes::from_str_name(&name);
            if let Some(id) = id {
                topics.push(Topics {
                    id: id as u32,
                    subclass_name: topic.subclass,
                });
            }
        }

        let (tx, rx) = mpsc::channel(10);
        let mut sub_rx = self.tx.subscribe();
        tokio::spawn(async move {
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
                            if topic.id == fsr::switch_event_types_t::SWITCH_EVENT_ALL.0 {
                                pass = true;
                                break;
                            } else if (topic.id == fsr::switch_event_types_t::SWITCH_EVENT_CUSTOM.0)
                                && (topic.subclass_name == e.subclass_name)
                            {
                                pass = true;
                                break;
                            } else if topic.id == e.event_id {
                                pass = true;
                                break;
                            }
                        }
                        if pass {
                            let send = tx.send(Ok(e)).await;

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
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// Command sends a single command to the server and returns a response Event.
    async fn command(&self, request: Request<CommandRequest>) -> Result<Response<Reply>, Status> {
        let req = request.into_inner();
        let mut cmd = req.command;
        let mut args = req.args;
        if cmd.contains("reload") && args.contains("mod_zrs") {
            cmd = String::from("bgapi");
            args = String::from("reload mod_zrs");
        } else if cmd.contains("unload") && args.contains("mod_zrs") {
            let reply = Reply {
                code: 501,
                message: String::from("-ERR Module mod_zrs is in use, cannot unload"),
            };
            return Ok(Response::new(reply));
        }

        let handle = tokio::task::spawn_blocking(move || fsr::api_exec(&cmd, &args));

        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// SendMsg sends messages to FreeSWITCH and returns a response.
    async fn send_msg(&self, request: Request<SendMsgRequest>) -> Result<Response<Reply>, Status> {
        let req = request.into_inner();
        let handle = tokio::task::spawn_blocking(move || fsr::sendmsg(&req.uuid, req.headers));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// SendEvent sends event to FreeSWITCH.
    async fn send_event(
        &self,
        request: Request<SendEventRequest>,
    ) -> Result<Response<Reply>, Status> {
        let req = request.into_inner();
        let handle = tokio::task::spawn_blocking(move || {
            fsr::sendevent(req.event_id, &req.subclass_name, req.headers, &req.body)
        });
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// reload xml
    async fn reload_xml(
        &self,
        _request: Request<ReloadXmlRequest>,
    ) -> Result<Response<Reply>, Status> {
        // let _req: ReloadXmlRequest = request.into_inner();
        let handle = tokio::task::spawn_blocking(|| fsr::api_exec("reloadxml", ""));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Reload acl
    async fn reload_acl(
        &self,
        _request: Request<ReloadAclRequest>,
    ) -> Result<Response<Reply>, Status> {
        // let _req = request.into_inner();
        let handle = tokio::task::spawn_blocking(|| fsr::api_exec("reloadacl", ""));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Reload mod
    async fn reload_mod(&self, request: Request<ModRequest>) -> Result<Response<Reply>, Status> {
        let req = request.into_inner();
        let mut cmd = "reload";
        let mut args = req.mod_name;

        if args.contains("mod_zrs") {
            cmd = "bgapi";
            args = String::from("reload mod_zrs");
        }

        let handle = tokio::task::spawn_blocking(move || fsr::api_exec(cmd, &args));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Load mod
    async fn load_mod(&self, request: Request<ModRequest>) -> Result<Response<Reply>, Status> {
        let req = request.into_inner();
        let handle = tokio::task::spawn_blocking(move || fsr::api_exec("load", &req.mod_name));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Unload mod
    async fn unload_mod(&self, request: Request<ModRequest>) -> Result<Response<Reply>, Status> {
        let req = request.into_inner();
        if req.mod_name.contains("mod_zrs") {
            let reply = Reply {
                code: 501,
                message: String::from("-ERR Module mod_zrs is in use, cannot unload"),
            };
            return Ok(Response::new(reply));
        }

        let handle = tokio::task::spawn_blocking(move || fsr::api_exec("unload", &req.mod_name));
        let res: Result<String, String> = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// JSAPI
    async fn jsapi(&self, request: Request<JsapiRequest>) -> Result<Response<Reply>, Status> {
        let req = request.into_inner();

        let handle = tokio::task::spawn_blocking(move || fsr::json_api_exec(&req.command));

        let res: Result<String, String> = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(message) => {
                let reply = Reply { code: 500, message };
                Ok(Response::new(reply))
            }
        }
    }
}
