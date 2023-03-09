use async_trait::async_trait;
use log::{debug, info, trace, warn};
use rhiaqey_sdk::message::MessageValue;
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::RwLock;
use ureq::{AgentBuilder, Request};

fn default_interval() -> Option<u64> {
    Some(15000)
}

fn default_timeout() -> Option<u64> {
    Some(5000)
}

fn default_endpoint() -> Option<String> {
    Some("http://api.open-notify.org/iss-now.json".to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct ISSPositionSettings {
    #[serde(alias = "Endpoint", default = "default_endpoint")]
    pub endpoint: Option<String>,

    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,

    #[serde(alias = "Timeout", default = "default_timeout")]
    pub timeout_in_millis: Option<u64>,
}

impl Default for ISSPositionSettings {
    fn default() -> Self {
        ISSPositionSettings {
            endpoint: default_endpoint(),
            interval_in_millis: default_interval(),
            timeout_in_millis: default_timeout(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct ISSPositionObject {
    pub latitude: String,
    pub longitude: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct ISSPositionResponse {
    pub iss_position: ISSPositionObject,
    pub timestamp: u64,
    pub message: String,
}

#[derive(Default, Debug)]
pub struct ISSPosition {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<RwLock<ISSPositionSettings>>,
}

impl ISSPosition {
    async fn get_request(&self) -> Request {
        let settings = self.settings.read().await;
        let endpoint = settings.endpoint.as_ref().unwrap();

        let agent = AgentBuilder::new()
            .timeout(Duration::from_millis(settings.timeout_in_millis.unwrap()))
            .build();

        debug!("downloading from {}", endpoint.clone());
        agent.get(endpoint.as_str())
    }

    async fn fetch_position(&self) -> Result<ISSPositionResponse, Box<dyn std::error::Error>> {
        info!("fetching position");

        let req = self.get_request().await;
        let res = req.call()?.into_json::<ISSPositionResponse>()?;

        debug!("iss position downloaded");

        Ok(res)
    }

    fn prepare_message(&self, payload: ISSPositionResponse) -> ProducerMessage {
        debug!("preparing message from response");

        let tag = Some(digest(format!(
            "{}-{}",
            payload.iss_position.latitude, payload.iss_position.longitude
        )));

        let timestamp = Some(payload.timestamp * 1000);

        let json = serde_json::to_value(payload).unwrap();

        ProducerMessage {
            key: String::from("iss/position"),
            value: MessageValue::Json(json),
            category: Some("default".to_string()),
            size: None,
            timestamp,
            tag,
        }
    }
}

#[async_trait]
impl Producer for ISSPosition {
    fn setup(&mut self, settings: Option<String>) -> ProducerMessageReceiver {
        info!("setting up {}", self.kind());

        self.settings = Arc::new(RwLock::new(match settings {
            None => ISSPositionSettings::default(),
            Some(result) => {
                serde_json::from_str(result.as_str()).unwrap_or(ISSPositionSettings::default())
            }
        }));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: String) {
        let mut locked_settings = self.settings.write().await;
        *locked_settings =
            serde_json::from_str(settings.as_str()).unwrap_or(ISSPositionSettings::default());
    }

    async fn start(&mut self) {
        info!("starting {}", self.kind());

        let sender = self.sender.clone().unwrap();

        loop {
            let settings = self.settings.read().await;
            let interval = settings.interval_in_millis;

            match self.fetch_position().await {
                Ok(response) => {
                    trace!("we have our response {:?}", response);
                    sender
                        .send(self.prepare_message(response))
                        .expect("failed to send message");
                    trace!("message sent");
                }
                Err(err) => warn!("error fetching feed: {}", err),
            }

            thread::sleep(Duration::from_millis(interval.unwrap()));
        }
    }

    fn kind(&self) -> String {
        "iss_position".to_string()
    }
}
