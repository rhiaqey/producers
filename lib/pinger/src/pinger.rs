use async_trait::async_trait;
use log::{debug, info};
use rhiaqey_sdk_rs::message::MessageValue;
use rhiaqey_sdk_rs::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use rhiaqey_sdk_rs::settings::Settings;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;

fn default_update_tag() -> Option<bool> {
    Some(true)
}

fn default_update_timestamp() -> Option<bool> {
    Some(true)
}

fn default_interval() -> Option<u64> {
    Some(1000)
}

#[derive(Deserialize, Clone, Debug)]
pub struct PingerSettings {
    #[serde(alias = "UpdateTags", default = "default_update_tag")]
    pub update_tag: Option<bool>,
    #[serde(alias = "UpdateTimestamp", default = "default_update_timestamp")]
    pub update_timestamp: Option<bool>,
    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,
}

impl Default for PingerSettings {
    fn default() -> Self {
        PingerSettings {
            update_tag: default_update_tag(),
            update_timestamp: default_update_timestamp(),
            interval_in_millis: default_interval(),
        }
    }
}

impl Settings for PingerSettings {
    //
}

#[derive(Default, Debug)]
pub struct Pinger {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<Mutex<PingerSettings>>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct PingerBody {
    data: String,
}

#[async_trait]
impl Producer<PingerSettings> for Pinger {
    fn setup(&mut self, settings: Option<PingerSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", Self::kind());

        self.settings = Arc::new(Mutex::new(settings.unwrap_or(PingerSettings::default())));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: PingerSettings) {
        let mut locked_settings = self.settings.lock().await;
        *locked_settings = settings;
        debug!("new settings updated");
    }

    async fn start(&mut self) {
        info!("starting {}", Self::kind());

        let sender = self.sender.clone().unwrap();
        let settings = self.settings.clone();

        let epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let mut now = epoch.as_millis();

        tokio::task::spawn(async move {
            loop {
                let settings = settings.lock().await.clone();
                let interval = settings.interval_in_millis;
                let mut tag = Some(String::from("pinger"));
                let epoch = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();

                if settings.update_timestamp.unwrap_or(true) {
                    now = epoch.as_millis();
                }

                if settings.update_tag.unwrap_or(true) {
                    tag = Some(format!("pinger-{}", epoch.as_millis()));
                }

                let json = serde_json::to_value(PingerBody {
                    data: String::from("pinger"),
                })
                .unwrap();

                sender
                    .send(ProducerMessage {
                        tag,
                        key: String::from("pinger"),
                        value: MessageValue::Json(json),
                        category: None, // will be treated as default
                        size: None,
                        timestamp: Option::from(now as u64),
                    })
                    .unwrap();

                tokio::time::sleep(Duration::from_millis(interval.unwrap())).await;
            }
        });
    }

    fn schema() -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "UpdateTags": {
                    "type": "boolean",
                    "examples": [ false ],
                },
                "UpdateTimestamp": {
                    "type": "boolean",
                    "examples": [ false ],
                },
                "Interval": {
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
        String::from("pinger")
    }
}
