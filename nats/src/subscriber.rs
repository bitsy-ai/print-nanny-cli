use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;

use anyhow::Result;
use clap::{crate_authors, Arg, ArgMatches, Command};
use futures::stream::StreamExt;
use log::{debug, error, info, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use printnanny_services::error::NatsError;
use printnanny_settings::sys_info;

use crate::error::RequestErrorMsg;

use super::message_v2::NatsRequestHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSubscriber<Request, Reply>
where
    Request: Serialize + DeserializeOwned + Debug + NatsRequestHandler,
    Reply: Serialize + DeserializeOwned + Debug,
{
    subject: String,
    nats_server_uri: String,
    hostname: String,
    require_tls: bool,
    workers: usize,
    nats_creds: Option<PathBuf>,
    _request: PhantomData<Request>,
    _response: PhantomData<Reply>,
}

const DEFAULT_NATS_SOCKET_PATH: &str = "/var/run/printnanny/nats-worker.sock";
const DEFAULT_NATS_URI: &str = "nats://localhost:4223";

pub const DEFAULT_NATS_EDGE_APP_NAME: &str = "nats-edge-worker";
pub const DEFAULT_NATS_EDGE_SUBJECT: &str = "pi.localhost.>";

pub fn get_default_nats_subject() -> String {
    let hostname = sys_info::hostname().unwrap();
    format!("pi.{}.>", hostname)
}

impl<Request, Reply> NatsSubscriber<Request, Reply>
where
    Request: Serialize
        + DeserializeOwned
        + Debug
        + NatsRequestHandler<Request = Request>
        + NatsRequestHandler<Reply = Reply>
        + std::marker::Sync
        + std::marker::Send,
    Reply: Serialize + DeserializeOwned + Debug + std::marker::Sync,
{
    pub fn clap_command(app_name: Option<String>) -> Command<'static> {
        let app_name = app_name.unwrap_or_else(|| DEFAULT_NATS_EDGE_APP_NAME.to_string());

        let app = Command::new(app_name)
            .author(crate_authors!())
            .about("Run NATS-based pub/sub workers")
            .arg(
                Arg::new("subject")
                    .long("subject")
                    .takes_value(true)
                    .default_value(DEFAULT_NATS_EDGE_SUBJECT),
            )
            .arg(
                Arg::new("nats_server_uri")
                    .long("nats-server-uri")
                    .takes_value(true)
                    .default_value(DEFAULT_NATS_URI),
            )
            .arg(Arg::new("hostname").long("hostname").takes_value(true))
            .arg(Arg::new("nats_creds").long("nats-creds").takes_value(true))
            .arg(
                Arg::new("workers")
                    .long("workers")
                    .takes_value(true)
                    .default_value("8"),
            )
            .arg(
                Arg::new("socket")
                    .long("socket")
                    .takes_value(true)
                    .default_value(DEFAULT_NATS_SOCKET_PATH),
            );
        app
    }

    pub fn new(args: &ArgMatches) -> Self {
        let default_nats_subject = get_default_nats_subject();

        let subject = args.value_of("subject").unwrap_or(&default_nats_subject);

        // check if uri requires tls
        let nats_server_uri: &str = args.value_of("nats_server_uri").unwrap_or(DEFAULT_NATS_URI);
        let require_tls = nats_server_uri.contains("tls");

        // if nats.creds available, initialize authenticated nats connection
        info!(
            "Attempting to initialize NATS connection to {}",
            nats_server_uri
        );

        let nats_creds = args.value_of("nats_creds");
        let nats_creds = nats_creds.map(PathBuf::from);

        let system_hostname = sys_info::hostname().unwrap_or_else(|_| "localhost".into());
        let hostname = args
            .value_of("hostname")
            .unwrap_or(&system_hostname)
            .to_string()
            // always subscribe to lowercased hostname pattern
            // see https://github.com/bitsy-ai/printnanny-os/issues/238
            .to_lowercase();
        let workers: usize = args.value_of_t("workers").unwrap_or(8);
        Self {
            hostname,
            subject: subject.to_string(),
            nats_server_uri: nats_server_uri.to_string(),
            nats_creds,
            require_tls,
            workers,
            _request: PhantomData,
            _response: PhantomData,
        }
    }
    pub async fn subscribe_nats_subject(&self) -> Result<()> {
        let mut nats_client: Option<async_nats::Client> = None;
        while nats_client.is_none() {
            match self.try_init_nats_client().await {
                Ok(nc) => {
                    nats_client = Some(nc);
                }
                Err(_) => {
                    warn!("Waiting for NATS server to be available");
                    sleep(Duration::from_millis(2000)).await;
                }
            }
        }
        warn!(
            "Subscribing to subect {} with nats client {:?}",
            self.subject, nats_client
        );
        let nats_client = nats_client.unwrap();
        let subscriber = nats_client.subscribe(self.subject.clone()).await.unwrap();
        warn!(
            "Listening on {} where subject={}",
            &self.nats_server_uri, &self.subject
        );

        subscriber
            .for_each_concurrent(self.workers, |message| async {
                let subject_pattern =
                    Request::replace_subject_pattern(&message.subject, &self.hostname, "{pi_id}");
                debug!(
                    "Extracted subject_pattern {} from subject {} using hostname {}",
                    &subject_pattern, &message.subject, &self.hostname
                );
                match Request::deserialize_payload(&subject_pattern, &message.payload) {
                    Ok(request) => {
                        debug!("Received NATS Message: {:?}", message);
                        if let Some(reply_inbox) = message.reply {
                            let payload = self.handle_request(request, &subject_pattern).await;
                            match &nats_client.publish(reply_inbox, payload.into()).await {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("Error publishing msg: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error deserializing NATS message: {}", e);
                    }
                }
            })
            .await;
        Ok(())
    }
    // FIFO buffer flush
    pub async fn try_flush_buffer(
        &self,
        request_buffer: &[(String, Vec<u8>)],
        nats_client: &async_nats::Client,
    ) -> Result<(), NatsError> {
        for request in request_buffer.iter() {
            let (subject, payload) = request;
            match nats_client
                .publish(subject.to_string(), payload.clone().into())
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(NatsError::PublishError {
                    error: e.to_string(),
                }),
            }?;
        }

        Ok(())
    }

    async fn handle_request(&self, request: Request, subject_pattern: &str) -> Vec<u8> {
        match request.handle().await {
            Ok(r) => serde_json::to_vec(&r).unwrap(),
            Err(e) => {
                let r = RequestErrorMsg {
                    error: e.to_string(),
                    subject_pattern: subject_pattern.to_string(),
                    request,
                };
                serde_json::to_vec(&r).unwrap()
            }
        }
    }

    pub async fn try_init_nats_client(&self) -> Result<async_nats::Client, std::io::Error> {
        match &self.nats_creds {
            Some(nats_creds) => match nats_creds.exists() {
                true => {
                    async_nats::ConnectOptions::with_credentials_file(nats_creds.clone())
                        .await?
                        .require_tls(self.require_tls)
                        .connect(&self.nats_server_uri)
                        .await
                }
                false => {
                    warn!(
                        "Failed to read {}. Initializing NATS client without credentials",
                        nats_creds.display()
                    );
                    async_nats::ConnectOptions::new()
                        .require_tls(self.require_tls)
                        .connect(&self.nats_server_uri)
                        .await
                }
            },
            None => {
                async_nats::ConnectOptions::new()
                    .require_tls(self.require_tls)
                    .connect(&self.nats_server_uri)
                    .await
            }
        }
    }
    pub async fn run(&self) -> Result<()> {
        self.subscribe_nats_subject().await?;
        Ok(())
    }
}
