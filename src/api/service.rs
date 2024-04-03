use fsr::*;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

fn to_struct(json: serde_json::Map<String, serde_json::Value>) -> prost_types::Struct {
    prost_types::Struct {
        fields: json
            .into_iter()
            .map(|(k, v)| (k, serde_json_to_prost(v)))
            .collect(),
    }
}

fn serde_json_to_prost(json: serde_json::Value) -> prost_types::Value {
    use prost_types::value::Kind::*;
    use serde_json::Value::*;
    prost_types::Value {
        kind: Some(match json {
            Null => NullValue(0 /* wat? */),
            Bool(v) => BoolValue(v),
            Number(n) => NumberValue(n.as_f64().unwrap_or(0.0)),
            String(s) => StringValue(s),
            Array(v) => ListValue(prost_types::ListValue {
                values: v.into_iter().map(serde_json_to_prost).collect(),
            }),
            Object(v) => StructValue(to_struct(v)),
        }),
    }
}

fn prost_to_serde_json(x: prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind::*;
    use serde_json::Value::*;
    match x.kind {
        Some(x) => match x {
            NullValue(_) => serde_json::json!(null),
            BoolValue(v) => Bool(v),
            NumberValue(n) => Number(serde_json::Number::from_f64(n).unwrap()),
            StringValue(s) => String(s),
            ListValue(lst) => Array(lst.values.into_iter().map(prost_to_serde_json).collect()),
            StructValue(v) => Object(
                v.fields
                    .into_iter()
                    .map(|(k, v)| (k, prost_to_serde_json(v)))
                    .collect(),
            ),
        },
        None => serde_json::json!(null),
    }
}

pub struct Service {
    pub tx: broadcast::Sender<super::zrapi::Event>,
}

struct Topics {
    id: u32,
    subclass_name: String,
}

#[tonic::async_trait]
impl super::zrapi::base_server::Base for Service {
    type SubscribeStream = ReceiverStream<Result<super::zrapi::Event, Status>>;

    /// Event Stream
    async fn subscribe(
        &self,
        request: Request<super::zrapi::SubscribeRequest>,
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
    async fn command(&self, request: Request<super::zrapi::CommandRequest>) -> Result<Response<super::zrapi::Reply>, Status> {
        let req = request.into_inner();
        let mut cmd = req.command;
        let mut args = req.args;

        let mut json_format = false;
        if cmd.find("json").is_some() || args.find("json").is_some() {
            json_format = true;
        }

        if cmd.contains("reload") && args.contains("mod_zrs") {
            cmd = String::from("bgapi");
            args = String::from("reload mod_zrs");
        } else if cmd.contains("unload") && args.contains("mod_zrs") {
            let reply = super::zrapi::Reply {
                code: 501,
                message: String::from("Module mod_zrs is in use, cannot unload"),
                data: None,
            };
            return Ok(Response::new(reply));
        }

        let handle = tokio::task::spawn_blocking(move || fsr::api_exec(&cmd, &args));

        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply =super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                if json_format {
                    let json_value: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_str(&msg);
                    if let Ok(json_value) = json_value {
                        let value =
                            serde_json::json!({"response": json_value, "status": "success"});
                        let response = serde_json_to_prost(value);
                        let reply = super::zrapi::Reply {
                            code: 200,
                            message: "OK".to_string(),
                            data: Some(response),
                        };
                        return Ok(Response::new(reply));
                    }
                }
                let reply = super::zrapi::Reply {
                    code: 200,
                    message: msg,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// SendMsg sends messages to FreeSWITCH and returns a response.
    async fn send_msg(&self, request: Request<super::zrapi::SendMsgRequest>) -> Result<Response<super::zrapi::Reply>, Status> {
        let req = request.into_inner();
        let handle = tokio::task::spawn_blocking(move || fsr::sendmsg(&req.uuid, req.headers));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::zrapi::Reply {
                    code: 200,
                    message: msg,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// SendEvent sends event to FreeSWITCH.
    async fn send_event(
        &self,
        request: Request<super::zrapi::SendEventRequest>,
    ) -> Result<Response<super::zrapi::Reply>, Status> {
        let req = request.into_inner();
        let handle = tokio::task::spawn_blocking(move || {
            fsr::sendevent(req.event_id, &req.subclass_name, req.headers, &req.body)
        });
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::zrapi::Reply {
                    code: 200,
                    message: msg,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// reload xml
    async fn reload_xml(
        &self,
        _request: Request<super::zrapi::ReloadXmlRequest>,
    ) -> Result<Response<super::zrapi::Reply>, Status> {
        // let _req: ReloadXmlRequest = request.into_inner();
        let handle = tokio::task::spawn_blocking(|| fsr::api_exec("reloadxml", ""));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::zrapi::Reply {
                    code: 200,
                    message: msg,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Reload acl
    async fn reload_acl(
        &self,
        _request: Request<super::zrapi::ReloadAclRequest>,
    ) -> Result<Response<super::zrapi::Reply>, Status> {
        // let _req = request.into_inner();
        let handle = tokio::task::spawn_blocking(|| fsr::api_exec("reloadacl", ""));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::zrapi::Reply {
                    code: 200,
                    message: msg,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Reload mod
    async fn reload_mod(&self, request: Request<super::zrapi::ModRequest>) -> Result<Response<super::zrapi::Reply>, Status> {
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
                let reply = super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::zrapi::Reply {
                    code: 200,
                    message: msg,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Load mod
    async fn load_mod(&self, request: Request<super::zrapi::ModRequest>) -> Result<Response<super::zrapi::Reply>, Status> {
        let req = request.into_inner();
        let handle = tokio::task::spawn_blocking(move || fsr::api_exec("load", &req.mod_name));
        let res = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::zrapi::Reply {
                    code: 200,
                    message: msg,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Unload mod
    async fn unload_mod(&self, request: Request<super::zrapi::ModRequest>) -> Result<Response<super::zrapi::Reply>, Status> {
        let req = request.into_inner();
        if req.mod_name.contains("mod_zrs") {
            let reply = super::zrapi::Reply {
                code: 501,
                message: String::from("-ERR Module mod_zrs is in use, cannot unload"),
                data: None,
            };
            return Ok(Response::new(reply));
        }

        let handle = tokio::task::spawn_blocking(move || fsr::api_exec("unload", &req.mod_name));
        let res: Result<String, String> = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::zrapi::Reply {
                    code: 200,
                    message: msg,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// JSAPI
    async fn jsapi(&self, request: Request<super::zrapi::JsapiRequest>) -> Result<Response<super::zrapi::Reply>, Status> {
        let req = request.into_inner();

        let args = req.args.unwrap_or_default();
        let args = prost_to_serde_json(args);

        let cmd = serde_json::json!({"data": args, "command": &req.command});
        let cmd = cmd.to_string();

        let mut json_format = false;
        if req.command.eq_ignore_ascii_case("fsapi") && cmd.find("json").is_some() {
            json_format = true;
        }

        let handle = tokio::task::spawn_blocking(move || fsr::json_api_exec(&cmd));
        let res: Result<String, String> = handle.await.unwrap();
        match res {
            Err(e) => {
                let reply = super::zrapi::Reply {
                    code: 500,
                    message: e,
                    data: None,
                };
                Ok(Response::new(reply))
            }
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
                                        let msg_value: Result<
                                            serde_json::Value,
                                            serde_json::Error,
                                        > = serde_json::from_str(msg);
                                        if let Ok(msg_value) = msg_value {
                                            object.remove("response");
                                            object.insert("response".to_string(), msg_value);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let reply = super::zrapi::Reply {
                        code: 200,
                        message: "OK".to_string(),
                        data: Some(serde_json_to_prost(json_value)),
                    };

                    return Ok(Response::new(reply));
                }
                let reply = super::zrapi::Reply {
                    code: 200,
                    message,
                    data: None,
                };
                Ok(Response::new(reply))
            }
        }
    }
}
