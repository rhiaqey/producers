use async_trait::async_trait;
use log::info;
use rhiaqey_sdk::message::MessageValue;
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::RwLock;

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

#[derive(Default, Debug)]
pub struct Ticker {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<RwLock<TickerSettings>>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct TickerBody {
    timestamp: u64,
}

#[async_trait]
impl Producer<TickerSettings> for Ticker {
    fn setup(&mut self, settings: Option<TickerSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", Ticker::kind());

        let settings = settings.unwrap_or(TickerSettings::default());
        self.settings = Arc::new(RwLock::new(settings));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn get_settings(&self) -> TickerSettings {
        self.settings.read().await.clone()
    }

    async fn set_settings(&mut self, settings: TickerSettings) {
        let mut locked_settings = self.settings.write().await;
        *locked_settings = settings;
    }

    async fn start(&mut self) {
        info!("starting {}", Self::kind());

        let sender = self.sender.clone().unwrap();

        loop {
            let settings = self.get_settings().await;
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
                    category: None,
                    size: None,
                    timestamp: Option::from(now as u64),
                })
                .unwrap();

            thread::sleep(Duration::from_millis(interval.unwrap()));
        }
    }

    fn kind() -> String {
        "ticker".to_string()
    }
}
