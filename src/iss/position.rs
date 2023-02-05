use log::{debug, info, trace, warn};
use rhiaqey_sdk::message::MessageValue;
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use ureq::{AgentBuilder, Request};

fn default_interval() -> u64 {
    5000
}

fn default_timeout() -> u64 {
    5000
}

fn default_endpoint() -> String { "http://api.open-notify.org/iss-now.json".to_string() }

#[derive(Default, Deserialize, Clone, Debug)]
pub struct ISSPositionSettings {
    #[serde(alias = "Endpoint", default="default_endpoint")]
    pub endpoint: String,
    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: u64,
    #[serde(alias = "Timeout", default = "default_timeout")]
    pub timeout_in_millis: u64,
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
    settings: Arc<Mutex<ISSPositionSettings>>,
}

impl ISSPosition {
    fn get_request(&self) -> Request {
        let settings = self.settings.lock().unwrap().clone();

        let agent = AgentBuilder::new()
            .timeout(Duration::from_millis(settings.timeout_in_millis))
            .build();

        agent.get(settings.endpoint.as_str())
    }

    fn fetch_position(&self) -> Result<ISSPositionResponse, Box<dyn std::error::Error>> {
        debug!("fetching feed");

        let req = self.get_request();
        let res = req.call()?.into_json::<ISSPositionResponse>()?;

        debug!("iss position downloaded");

        Ok(res)
    }

    fn prepare_message(&self, payload: ISSPositionResponse) -> ProducerMessage {
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

impl Producer<ISSPositionSettings> for ISSPosition {
    fn setup(&mut self, settings: Option<ISSPositionSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", Self::kind());

        let settings = settings.unwrap_or(ISSPositionSettings::default());
        self.settings = Arc::new(Mutex::new(settings));
        debug!("settings parsed {:?}", self.settings);

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    fn start(&self) {
        info!("starting {}", Self::kind());

        let interval = self.settings.lock().unwrap().interval_in_millis;
        let sender = self.sender.clone().unwrap();

        loop {
            match self.fetch_position() {
                Ok(response) => {
                    trace!("we have our response {:?}", response);
                    sender
                        .send(self.prepare_message(response))
                        .expect("failed to send message");
                }
                Err(err) => warn!("error fetching feed: {}", err),
            }

            thread::sleep(Duration::from_millis(interval));
        }
    }

    fn kind() -> String {
        "iss_position".to_string()
    }
}
