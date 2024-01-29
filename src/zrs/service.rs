use fsr::*;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

pub struct Service {
    pub tx: broadcast::Sender<super::Event>,
}

#[tonic::async_trait]
impl super::zrs_server::Zrs for Service {
    type EventStream = ReceiverStream<Result<super::EventReply, Status>>;

    async fn event(
        &self,
        request: Request<super::EventRequest>,
    ) -> Result<Response<Self::EventStream>, Status> {
        let mut remote_addr_str = String::from("");
        let remote_addr = request.remote_addr();

        if let Some(remote_addr) = remote_addr {
            remote_addr_str = remote_addr.to_string();
        }

        info!("Got a subscriber from {}", remote_addr_str);
        let topics = request.into_inner().topics;

        let (tx, rx) = mpsc::channel(10);
        let mut sub_rx = self.tx.subscribe();
        tokio::spawn(async move {
            let mut seq = 0u64;
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
                            if topic.id == super::EventTypes::SwitchEventAll as i32 {
                                pass = true;
                                break;
                            } else if (topic.id == super::EventTypes::SwitchEventCustom as i32)
                                && (topic.subclass == e.subclass_name)
                            {
                                pass = true;
                                break;
                            } else if topic.id == e.event_id as i32 {
                                pass = true;
                                break;
                            }
                        }
                        if pass {
                            let send = tx
                                .send(Ok(super::EventReply {
                                    seq,
                                    event: Some(e),
                                }))
                                .await;

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

                debug!("Send Event SEQ: {:?}", seq);
                seq = seq + 1;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// Command sends a single command to the server and returns a response Event.
    async fn command(
        &self,
        request: Request<super::CommandRequest>,
    ) -> Result<Response<super::Reply>, Status> {
        let req = request.into_inner();
        debug!("{:?}", req);
        let reply = super::Reply {
            code: 200,
            message: String::from("OK"),
        };
        Ok(Response::new(reply))
    }

    /// SendMsg sends messages to FreeSWITCH and returns a response Event.
    async fn send_msg(
        &self,
        request: Request<super::SendMsgRequest>,
    ) -> Result<Response<super::Reply>, Status> {
        let req = request.into_inner();
        debug!("{:?}", req);
        let reply = super::Reply {
            code: 200,
            message: String::from("OK"),
        };
        Ok(Response::new(reply))
    }

    // reload xml
    async fn reload_xml(
        &self,
        request: Request<super::ReloadXmlRequest>,
    ) -> Result<Response<super::Reply>, Status> {
        let _req: super::ReloadXmlRequest = request.into_inner();
        // debug!("{:?}", req);
        let ret = fsr::api_exec("reloadxml", "");
        match ret {
            Err(e) => {
                let reply = super::Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    // reload acl
    async fn reload_acl(
        &self,
        request: Request<super::ReloadAclRequest>,
    ) -> Result<Response<super::Reply>, Status> {
        let _req = request.into_inner();
        // debug!("{:?}", req);
        let ret = fsr::api_exec("reloadacl", "");
        match ret {
            Err(e) => {
                let reply = super::Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Reload mod
    async fn reload_mod(
        &self,
        request: Request<super::ModRequest>,
    ) -> Result<Response<super::Reply>, Status> {
        let req = request.into_inner();
        // debug!("{:?}", req);
        let ret = fsr::api_exec("reload", req.mod_name.as_str());
        match ret {
            Err(e) => {
                let reply = super::Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Load mod
    async fn load_mod(
        &self,
        request: Request<super::ModRequest>,
    ) -> Result<Response<super::Reply>, Status> {
        let req = request.into_inner();
        // debug!("{:?}", req);
        let ret = fsr::api_exec("load", req.mod_name.as_str());
        match ret {
            Err(e) => {
                let reply = super::Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }

    /// Unload mod
    async fn unload_mod(
        &self,
        request: Request<super::ModRequest>,
    ) -> Result<Response<super::Reply>, Status> {
        let req = request.into_inner();
        // debug!("{:?}", req);
        let ret = fsr::api_exec("unload", req.mod_name.as_str());
        match ret {
            Err(e) => {
                let reply = super::Reply {
                    code: 500,
                    message: e,
                };
                Ok(Response::new(reply))
            }
            Ok(msg) => {
                let reply = super::Reply {
                    code: 200,
                    message: msg,
                };
                Ok(Response::new(reply))
            }
        }
    }
}
