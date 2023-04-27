use async_trait::async_trait;
use chrono::prelude::*;
use futures::TryFutureExt;
use log::{debug, info, trace, warn};
use quick_xml::de::from_str;
use reqwest::Response;
use rhiaqey_common::error::RhiaqeyError;
use rhiaqey_sdk::message::MessageValue;
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
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
    Some("https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml".to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct ECBDailySettings {
    #[serde(alias = "Url", default = "default_url")]
    pub url: Option<String>,

    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,

    #[serde(alias = "Timeout", default = "default_timeout")]
    pub timeout_in_millis: Option<u64>,
}

impl Default for ECBDailySettings {
    fn default() -> Self {
        ECBDailySettings {
            url: default_url(),
            interval_in_millis: default_interval(),
            timeout_in_millis: default_timeout(),
        }
    }
}

#[derive(Default, Debug)]
pub struct ECBDaily {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<Mutex<ECBDailySettings>>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ECBDailyResponseEnvelopeCubeWithCurrencyAndRate {
    #[serde(rename = "@currency")]
    pub currency: String,
    #[serde(rename = "@rate")]
    pub rate: f64,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ECBDailyResponseEnvelopeCubeWithTime {
    #[serde(rename = "@time")]
    pub time: String,
    #[serde(rename = "Cube")]
    pub cube: Vec<ECBDailyResponseEnvelopeCubeWithCurrencyAndRate>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ECBDailyResponseEnvelopeCube {
    #[serde(rename = "Cube")] // Cube > Cube
    pub cube: Vec<ECBDailyResponseEnvelopeCubeWithTime>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ECBDailyResponseEnvelopeSender {
    pub name: String, // European Central Bank
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename = "Envelope")]
pub struct ECBDailyResponse {
    pub subject: String, // Reference rates
    #[serde(rename = "Sender")]
    pub sender: ECBDailyResponseEnvelopeSender,
    #[serde(rename = "Cube")]
    pub cube: ECBDailyResponseEnvelopeCube,
}

#[derive(Default, Serialize, Clone, Debug, PartialEq)]
pub struct ECBDailyRate {
    pub symbol: String,
    pub amount: f64,
    pub timestamp: u64,
}

impl ECBDaily {
    async fn send_request(settings: ECBDailySettings) -> Result<Response, RhiaqeyError> {
        info!("fetching ecb daily");

        let client = reqwest::Client::new();
        let timeout = settings
            .timeout_in_millis
            .unwrap_or(default_timeout().unwrap());

        if settings.url.is_none() {
            return Err(RhiaqeyError {
                code: None,
                message: String::from("url is not configured properly"),
                error: None,
            });
        }

        client
            .get(settings.url.unwrap())
            .timeout(Duration::from_millis(timeout))
            .send()
            .map_err(|x| x.into())
            .await
    }

    async fn fetch_daily_rates(
        settings: ECBDailySettings,
    ) -> Result<ECBDailyResponse, RhiaqeyError> {
        info!("downloading daily rates");

        let res = Self::send_request(settings).await?;
        let text = res.text().await?;
        let daily = from_str(text.as_str())?;
        debug!("ecb daily downloaded");

        Ok(daily)
    }

    fn prepare_daily_rates(payload: ECBDailyResponse) -> Vec<ProducerMessage> {
        debug!("preparing messages from response");

        let mut messages: Vec<ProducerMessage> = vec![];

        payload.cube.cube.iter().for_each(|x| {
            if let Ok(tms) =
                Utc.datetime_from_str(format!("{} 00:00:00", x.time).as_str(), "%Y-%m-%d %H:%M:%S")
            {
                x.cube.iter().for_each(|y| {
                    let rate = ECBDailyRate {
                        symbol: format!("EUR{}", y.currency),
                        amount: y.rate,
                        timestamp: tms.timestamp_millis() as u64,
                    };

                    let data_value = serde_json::to_value(&rate).unwrap();

                    if let Ok(raw) = serde_json::to_string(&rate) {
                        let tag = sha256::digest(raw);
                        messages.push(ProducerMessage {
                            size: Some(1),
                            key: rate.symbol,
                            category: None,
                            value: MessageValue::Json(data_value),
                            tag: Some(tag),
                            timestamp: Some(rate.timestamp),
                        })
                    }
                });
            }
        });

        messages
    }
}

#[async_trait]
impl Producer<ECBDailySettings> for ECBDaily {
    fn setup(&mut self, settings: Option<ECBDailySettings>) -> ProducerMessageReceiver {
        info!("setting up {}", self.kind());

        self.settings = Arc::new(Mutex::new(settings.unwrap_or(ECBDailySettings::default())));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: ECBDailySettings) {
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

                if settings.url.is_some() {
                    match Self::fetch_daily_rates(settings).await {
                        Ok(response) => {
                            trace!("we have our response");

                            for item in Self::prepare_daily_rates(response) {
                                sender.send(item).expect("failed to send message");
                            }

                            trace!("rates sent");
                        }
                        Err(err) => warn!("error fetching feed: {}", err),
                    }
                }

                thread::sleep(Duration::from_millis(interval.unwrap()));
            }
        });
    }

    fn kind(&self) -> String {
        String::from("ecb_daily")
    }
}