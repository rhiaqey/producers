use base64::{engine::general_purpose, Engine as _};
use futures::stream::SplitSink;
use futures::SinkExt;
use futures::StreamExt;
use log::{debug, info, trace, warn};
use rhiaqey_sdk_rs::message::MessageValue;
use rhiaqey_sdk_rs::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use rhiaqey_sdk_rs::settings::Settings;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use yahoo_definitions::PricingData;

mod yahoo_definitions {
    include!(concat!(env!("OUT_DIR"), "/yahoo_realtime.rs"));
}

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

impl Settings for YahooSettings {
    //
}

#[derive(Default, Debug)]
pub struct Yahoo {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<Mutex<YahooSettings>>,
    writer: Option<Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
enum YahooStreamAction {
    Subscribe(HashSet<String>),
    Unsubscribe(HashSet<String>),
}

impl Producer<YahooSettings> for Yahoo {
    fn setup(&mut self, settings: Option<YahooSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", Self::kind());

        self.settings = Arc::new(Mutex::new(settings.unwrap_or(YahooSettings::default())));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: YahooSettings) {
        let mut locked_settings = self.settings.lock().await;

        if self.writer.is_some() {
            let unsubscription_message = serde_json::to_string(&YahooStreamAction::Unsubscribe(
                locked_settings.symbols.clone(),
            ))
            .unwrap();

            trace!("unsubscribing {}", unsubscription_message.clone());

            self.writer
                .as_mut()
                .unwrap()
                .lock()
                .await
                .send(Message::text(unsubscription_message))
                .await
                .unwrap();

            let subscription_message =
                serde_json::to_string(&YahooStreamAction::Subscribe(settings.symbols.clone()))
                    .unwrap();

            trace!("subscribing {}", subscription_message.clone());

            self.writer
                .as_mut()
                .unwrap()
                .lock()
                .await
                .send(Message::text(subscription_message))
                .await
                .unwrap();
        }

        *locked_settings = settings;
        debug!("new settings updated");
    }

    async fn start(&mut self) {
        info!("starting {}", Self::kind());

        let sender = self.sender.clone().unwrap();
        let settings = self.settings.lock().await.clone();

        let url = settings.url.unwrap_or(default_url().unwrap());

        let (socket, _) = connect_async(url).await.unwrap();

        let (mut ws_sender, mut wss_receiver) = socket.split();

        tokio::task::spawn(async move {
            // let sender = sender.clone();
            loop {
                tokio::select! {
                    Some(message) = wss_receiver.next() => {
                        match message.unwrap() {
                            Message::Ping(_) => {
                                debug!("pinger message arrived");
                            },
                            Message::Close(_) => {
                                debug!("close message arrived");
                            }
                            Message::Text(value) => {
                                trace!("text message arrived {}", value);

                                match general_purpose::STANDARD.decode(value) {
                                    Ok(raw) => {
                                        let data: PricingData = prost::Message::decode(raw.as_slice()).unwrap();
                                        let id = data.id.clone();
                                        let tag = format!("{}", data.time);
                                        let json = serde_json::to_value(data).unwrap();
                                        let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                                        let now = epoch.as_millis();

                                        sender
                                            .send(ProducerMessage {
                                                size: None,
                                                key: id,
                                                tag: Some(tag),
                                                value: MessageValue::Json(json),
                                                category: None, // will be treated as default
                                                timestamp: Option::from(now as u64),
                                            })
                                            .unwrap();
                                    },
                                    Err(err) => {
                                        warn!("error base64 decoding {}", err);
                                    },
                                }
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

        trace!("subscribing {}", subscription_message.clone());

        ws_sender
            .send(Message::text(subscription_message))
            .await
            .unwrap();

        self.writer = Some(Arc::new(Mutex::new(ws_sender)));
    }

    fn schema() -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "Url": {
                    "type": "string"
                },
                "Symbols": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
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
        String::from("yahoo")
    }
}
