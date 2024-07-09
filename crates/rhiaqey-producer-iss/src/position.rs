use anyhow::{bail, Context};
use log::{debug, info, trace, warn};
use reqwest::Response;
use rhiaqey_sdk_rs::message::MessageValue;
use rhiaqey_sdk_rs::producer::{
    Producer, ProducerConfig, ProducerMessage, ProducerMessageReceiver,
};
use rhiaqey_sdk_rs::settings::Settings;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha256::digest;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;

fn default_interval() -> Option<u64> {
    Some(15000)
}

fn default_timeout() -> Option<u64> {
    Some(5000)
}

fn default_url() -> Option<String> {
    Some("http://api.open-notify.org/iss-now.json".to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct ISSPositionSettings {
    #[serde(alias = "Url", default = "default_url")]
    pub url: Option<String>,

    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,

    #[serde(alias = "Timeout", default = "default_timeout")]
    pub timeout_in_millis: Option<u64>,
}

impl Default for ISSPositionSettings {
    fn default() -> Self {
        ISSPositionSettings {
            url: default_url(),
            interval_in_millis: default_interval(),
            timeout_in_millis: default_timeout(),
        }
    }
}

impl Settings for ISSPositionSettings {
    //
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
struct ISSPositionObject {
    pub latitude: String,
    pub longitude: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
struct ISSPositionResponse {
    pub iss_position: ISSPositionObject,
    pub timestamp: u64,
    pub message: String,
}

#[derive(Debug)]
pub struct ISSPosition {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<Mutex<ISSPositionSettings>>,
}

impl ISSPosition {
    async fn send_request(settings: ISSPositionSettings) -> anyhow::Result<Response> {
        info!("fetching iss position");

        let client = reqwest::Client::new();
        let timeout = settings
            .timeout_in_millis
            .unwrap_or(default_timeout().unwrap());

        if settings.url.is_none() {
            bail!("url is not configured properly")
        }

        client
            .get(settings.url.unwrap())
            .timeout(Duration::from_millis(timeout))
            .send()
            .await
            .context("failed to send request")
    }

    async fn fetch_position(settings: ISSPositionSettings) -> anyhow::Result<ISSPositionResponse> {
        info!("downloading iss position");

        let res = Self::send_request(settings)
            .await
            .context("failed to send request")?;

        let text = res
            .text()
            .await
            .context("failed to get full response as text")?;

        let position = serde_json::from_str::<ISSPositionResponse>(text.as_str())
            .context("failed to deserialize response")?;

        debug!("iss position downloaded");

        Ok(position)
    }

    fn prepare_message(payload: ISSPositionResponse) -> ProducerMessage {
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
            category: Some(String::from("position")),
            size: None,
            timestamp,
            tag,
            user_ids: None,
            client_ids: None,
        }
    }
}

impl Default for ISSPosition {
    fn default() -> Self {
        Self {
            sender: None,
            settings: Default::default(),
        }
    }
}

impl Producer<ISSPositionSettings> for ISSPosition {
    async fn setup(
        &mut self,
        _config: ProducerConfig,
        settings: Option<ISSPositionSettings>,
    ) -> ProducerMessageReceiver {
        info!("setting up {}", Self::kind());

        self.settings = Arc::new(Mutex::new(
            settings.unwrap_or(ISSPositionSettings::default()),
        ));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: ISSPositionSettings) {
        let mut locked_settings = self.settings.lock().await;
        *locked_settings = settings;
        debug!("new settings updated");
    }

    async fn start(&mut self) {
        info!("starting {}", Self::kind());

        let sender = self.sender.clone().unwrap();
        let settings = self.settings.clone();

        tokio::task::spawn(async move {
            loop {
                let settings = settings.lock().await.clone();
                let interval = settings.interval_in_millis;

                if settings.url.is_some() {
                    match Self::fetch_position(settings).await {
                        Ok(response) => {
                            trace!("we have our response {:?}", response);
                            sender
                                .send(Self::prepare_message(response))
                                .expect("failed to send message");
                            trace!("message sent");
                        }
                        Err(err) => warn!("error fetching feed: {}", err),
                    }
                }

                tokio::time::sleep(Duration::from_millis(interval.unwrap())).await;
            }
        });
    }

    fn schema() -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "Url": {
                    "type": "string",
                    "examples": [ "http://api.open-notify.org/iss-now.json" ],
                },
                "Interval": {
                    "type": "integer",
                    "examples": [ 5000 ],
                    "minimum": 1000
                },
                "Timeout": {
                    "type": "integer",
                    "examples": [ 15000 ],
                    "minimum": 1000
                }
            },
            "required": [],
            "additionalProperties": false
        })
    }

    async fn metrics(&self) -> Value {
        json!({})
    }

    fn kind() -> String {
        String::from("iss_position")
    }
}
