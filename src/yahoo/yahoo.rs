use async_trait::async_trait;
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

fn default_url() -> Option<String> {
    Some("wss://streamer.finance.yahoo.com".to_string())
}

#[derive(Default, Deserialize, Clone, Debug)]
pub struct YahooSettings {
    #[serde(alias = "Url", default = "default_url")]
    pub url: Option<String>,

    #[serde(alias = "Symbols")]
    symbols: HashSet<String>,
}

#[derive(Default, Debug)]
pub struct Yahoo {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<Mutex<YahooSettings>>,
}

#[async_trait]
impl Producer<YahooSettings> for Yahoo {
    fn setup(&mut self, _settings: Option<YahooSettings>) -> ProducerMessageReceiver {
        todo!()
    }
    async fn set_settings(&mut self, _settings: YahooSettings) {
        todo!()
    }
    async fn start(&mut self) {
        todo!()
    }
    fn kind(&self) -> String {
        "yahoo".into()
    }
}
