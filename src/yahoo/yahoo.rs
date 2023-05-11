use async_trait::async_trait;
use base64::decode;
use futures::SinkExt;
use futures::StreamExt;
use log::{debug, info, trace};
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

// https://github.com/fbriden/yahoo-finance-rs/blob/master/src/streaming.rs

fn default_url() -> Option<String> {
    Some(String::from("wss://streamer.finance.yahoo.com"))
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
enum YahooStreamAction {
    Subscribe(HashSet<String>),
}

#[async_trait]
impl Producer<YahooSettings> for Yahoo {
    fn setup(&mut self, settings: Option<YahooSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", self.kind());

        self.settings = Arc::new(Mutex::new(settings.unwrap_or(YahooSettings::default())));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: YahooSettings) {
        let mut locked_settings = self.settings.lock().await;
        *locked_settings = settings;
        debug!("new settings updated");
    }

    async fn start(&mut self) {
        info!("starting {}", self.kind());

        // let sender = self.sender.clone().unwrap();
        let settings = self.settings.lock().await.clone();

        let url = settings.url.unwrap_or(default_url().unwrap());

        let (socket, _) = connect_async(url).await.unwrap();

        let (mut ws_sender, mut wss_receiver) = socket.split();

        tokio::task::spawn(async move {
            loop {
                tokio::select! {
                    Some(message) = wss_receiver.next() => {
                        match message.unwrap() {
                            Message::Ping(_) => {
                                debug!("ping message arrived");
                            },
                            Message::Close(_) => {
                                debug!("close message arrived");
                            }
                            Message::Text(value) => {
                                debug!("text message arrived {}", value);
                                let data = parse_from_bytes::<PricingData>(&decode(msg).unwrap()).unwrap();
                            },
                            Message::Binary(value) => {
                                debug!("binary message arrived {:?}", value);
                            },
                            _ => {
                                trace!("unhandled message type");
                            }
                        }
                    }
                }
            }
        });

        // send the symbols we are interested in streaming
        let subscription_message =
            serde_json::to_string(&YahooStreamAction::Subscribe(settings.symbols.clone())).unwrap();

        debug!("sending sub message {}", subscription_message);

        ws_sender
            .send(Message::text(subscription_message))
            .await
            .unwrap()
    }

    fn kind(&self) -> String {
        String::from("yahoo")
    }
}
