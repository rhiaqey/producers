use log::info;
use rhiaqey_sdk::message::MessageValue;
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

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
    settings: Arc<Mutex<TickerSettings>>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct TickerBody {
    timestamp: u64,
}

impl Producer<TickerSettings> for Ticker {
    fn setup(&mut self, settings: Option<TickerSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", Ticker::kind());

        let settings = settings.unwrap_or(TickerSettings::default());
        self.settings = Arc::new(Mutex::new(settings));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    fn start(&self) {
        info!("starting {}", Ticker::kind());

        let interval = self.settings.lock().unwrap().interval_in_millis.unwrap();
        let sender = self.sender.clone().unwrap();

        loop {
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

            thread::sleep(Duration::from_millis(interval));
        }
    }

    fn kind() -> String {
        "ticker".to_string()
    }
}
