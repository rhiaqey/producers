use futures::TryFutureExt;
use log::{debug, info, trace, warn};
use reqwest::Response;
use rhiaqey_sdk_rs::message::MessageValue;
use rhiaqey_sdk_rs::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use rhiaqey_sdk_rs::settings::Settings;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha256::digest;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;

fn default_interval() -> Option<u64> {
    Some(900000)
}

fn default_timeout() -> Option<u64> {
    Some(5000)
}

fn default_url() -> Option<String> {
    Some("http://api.open-notify.org/astros.json".to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct ISSAstrosSettings {
    #[serde(alias = "Url", default = "default_url")]
    pub url: Option<String>,

    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,

    #[serde(alias = "Timeout", default = "default_timeout")]
    pub timeout_in_millis: Option<u64>,
}

impl Default for ISSAstrosSettings {
    fn default() -> Self {
        ISSAstrosSettings {
            url: default_url(),
            interval_in_millis: default_interval(),
            timeout_in_millis: default_timeout(),
        }
    }
}

impl Settings for ISSAstrosSettings {
    //
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
struct ISSAstrosPerson {
    pub craft: String,
    pub name: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
struct ISSAstrosResponse {
    pub people: Vec<ISSAstrosPerson>,
    pub number: u32,
    pub message: String,
}

#[derive(Default, Debug)]
pub struct ISSAstros {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<Mutex<ISSAstrosSettings>>,
}

impl ISSAstros {
    async fn send_request(settings: ISSAstrosSettings) -> Result<Response, String> {
        info!("fetching iss astros");

        let client = reqwest::Client::new();
        let timeout = settings
            .timeout_in_millis
            .unwrap_or(default_timeout().unwrap());

        if settings.url.is_none() {
            return Err(String::from("url is not configured properly"));
        }

        client
            .get(settings.url.unwrap())
            .timeout(Duration::from_millis(timeout))
            .send()
            .map_err(|x| x.to_string())
            .await
    }

    async fn fetch_astros(settings: ISSAstrosSettings) -> Result<ISSAstrosResponse, String> {
        info!("downloading iss astros");

        let res = Self::send_request(settings).await?;
        let text = res.text().await.map_err(|x| x.to_string())?;
        let astros =
            serde_json::from_str::<ISSAstrosResponse>(text.as_str()).map_err(|x| x.to_string())?;
        debug!("iss astros downloaded");

        Ok(astros)
    }

    fn prepare_message(payload: ISSAstrosResponse) -> ProducerMessage {
        debug!("preparing message from response");

        let epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let timestamp = Some(epoch.as_secs());

        let tag = Some(digest(format!("{}-{}", timestamp.unwrap(), payload.number)));

        let json = serde_json::to_value(payload).unwrap();

        ProducerMessage {
            key: String::from("iss/astros"),
            value: MessageValue::Json(json),
            category: Some(String::from("astros")),
            size: None,
            timestamp,
            tag,
            user_ids: None,
            client_ids: None,
            group_ids: None,
        }
    }
}

impl Producer<ISSAstrosSettings> for ISSAstros {
    fn setup(&mut self, settings: Option<ISSAstrosSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", Self::kind());

        self.settings = Arc::new(Mutex::new(settings.unwrap_or(ISSAstrosSettings::default())));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: ISSAstrosSettings) {
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
                    match Self::fetch_astros(settings).await {
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
                    "examples": [ "http://api.open-notify.org/astros.json" ],
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
        String::from("iss_astros")
    }
}
