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

fn default_interval() -> Option<u64> {
    Some(1000)
}

#[derive(Deserialize, Clone, Debug)]
pub struct TickerSettings {
    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,
}

impl Default for TickerSettings {
    fn default() -> Self {
        TickerSettings {
            interval_in_millis: default_interval(),
        }
    }
}

impl Settings for TickerSettings {
    //
}

#[derive(Default, Debug)]
pub struct Ticker {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<Mutex<TickerSettings>>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
struct TickerBody {
    timestamp: u64,
}

impl Producer<TickerSettings> for Ticker {
    fn setup(&mut self, settings: Option<TickerSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", Self::kind());

        self.settings = Arc::new(Mutex::new(settings.unwrap_or(TickerSettings::default())));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: TickerSettings) {
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

                let epoch = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();

                let now = epoch.as_millis();

                let json = serde_json::to_value(TickerBody {
                    timestamp: now as u64,
                })
                .unwrap();

                sender
                    .send(ProducerMessage {
                        tag: Some(format!("{now}")),
                        key: String::from("timestamp"),
                        value: MessageValue::Json(json),
                        category: None, // will be treated as default
                        size: None,
                        timestamp: Option::from(now as u64),
                        user_ids: None,
                        client_ids: None,
                        group_ids: None,
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
                "Interval": {
                    "type": "integer",
                    "examples": [ 5000 ],
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
        String::from("ticker")
    }
}
