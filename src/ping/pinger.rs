use async_trait::async_trait;
use log::{debug, info};
use rhiaqey_sdk::message::MessageValue;
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;

fn default_interval() -> Option<u64> {
    Some(1000)
}

#[derive(Deserialize, Clone, Debug)]
pub struct PingerSettings {
    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,
}

impl Default for PingerSettings {
    fn default() -> Self {
        PingerSettings {
            interval_in_millis: default_interval(),
        }
    }
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
        info!("setting up {}", self.kind());

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
        info!("starting {}", self.kind());

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

                let json = serde_json::to_value(PingerBody {
                    data: String::from("ping"),
                })
                .unwrap();

                sender
                    .send(ProducerMessage {
                        tag: Some(String::from("ping")),
                        key: String::from("ping"),
                        value: MessageValue::Json(json),
                        category: None, // will be treated as default
                        size: None,
                        timestamp: Option::from(now as u64),
                    })
                    .unwrap();

                thread::sleep(Duration::from_millis(interval.unwrap()));
            }
        });
    }

    fn kind(&self) -> String {
        "pinger".to_string()
    }
}
