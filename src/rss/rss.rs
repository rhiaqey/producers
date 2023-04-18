use async_trait::async_trait;
use chrono::DateTime;
use futures::TryFutureExt;
use log::{debug, info, trace, warn};
use reqwest::Response;
use rhiaqey_common::error::RhiaqeyError;
use rhiaqey_sdk::message::MessageValue;
use rhiaqey_sdk::producer::{Producer, ProducerMessage, ProducerMessageReceiver};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::RwLock;

fn default_url() -> Option<String> {
    None
}

fn default_username() -> Option<String> {
    None
}

fn default_password() -> Option<String> {
    None
}

fn default_interval() -> Option<u64> {
    Some(15000)
}

fn default_timeout() -> Option<u64> {
    Some(5000)
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct RSSResponse {
    #[serde(rename = "channel", skip_serializing_if = "Option::is_none")]
    pub channel: Option<rss::Channel>,

    #[serde(rename = "item", skip_serializing_if = "Option::is_none")]
    pub item: Option<rss::Item>,
}

impl RSSResponse {
    fn create(channel: Option<rss::Channel>, item: Option<rss::Item>) -> RSSResponse {
        RSSResponse { channel, item }
    }
    fn to_json(&self) -> Result<Value, RhiaqeyError> {
        serde_json::to_value(self).map_err(|x| x.into())
    }
}

#[derive(Default, Deserialize, Clone, Debug)]
pub struct RSSSettings {
    #[serde(alias = "Url", default = "default_url")]
    pub url: Option<String>,

    #[serde(alias = "Username", default = "default_username")]
    pub username: Option<String>,

    #[serde(alias = "Password", default = "default_password")]
    pub password: Option<String>,

    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,

    #[serde(alias = "Timeout", default = "default_timeout")]
    pub timeout_in_millis: Option<u64>,
}

#[derive(Default, Debug)]
pub struct RSS {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<RwLock<RSSSettings>>,
}

impl RSS {
    async fn send_request(settings: RSSSettings) -> Result<Response, RhiaqeyError> {
        info!("fetching feed");

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

        if let Some(username) = settings.username {
            return client
                .get(settings.url.unwrap())
                .timeout(Duration::from_millis(timeout))
                .basic_auth(username, settings.password)
                .send()
                .map_err(|x| x.into())
                .await;
        }

        client
            .get(settings.url.unwrap())
            .timeout(Duration::from_millis(timeout))
            .send()
            .map_err(|x| x.into())
            .await
    }

    async fn fetch_feed(settings: RSSSettings) -> Result<rss::Channel, RhiaqeyError> {
        info!("downloading rss feed");

        let res = Self::send_request(settings).await?;
        let text = res.text().await?;
        let data = text.as_bytes();
        let channel = rss::Channel::read_from(data)?;
        debug!("channel {} downloaded", channel.title);

        Ok(channel)
    }

    #[inline(always)]
    fn build_timestamp(date: String) -> i64 {
        if let Ok(timestamp) = DateTime::parse_from_rfc2822(date.as_str()) {
            return timestamp.timestamp_millis();
        }

        DateTime::parse_from_rfc3339(date.as_str())
            .unwrap()
            .timestamp_millis()
    }

    fn prepare_items(mut items: Vec<rss::Item>) -> Result<Vec<ProducerMessage>, RhiaqeyError> {
        items.sort_by(|a, b| {
            let timestamp_a =
                Self::build_timestamp(a.pub_date.clone().expect("a valid XML timestamp"));
            let timestamp_b =
                Self::build_timestamp(b.pub_date.clone().expect("a valid XML timestamp"));

            let result = timestamp_a.cmp(&timestamp_b);

            if result == Ordering::Equal {
                let title_a = a.title.as_ref().expect("could not find title");
                let title_b = b.title.as_ref().expect("could not find title");
                return title_a.cmp(title_b);
            }

            result
        });

        let result = items
            .into_iter()
            .map(|x| {
                let title = x.title.as_ref().unwrap().clone();

                let timestamp =
                    Self::build_timestamp(x.pub_date.clone().expect("a valid XML timestamp"));

                trace!("found rss entry {}: {}", timestamp, title);

                let tag = sha256::digest(format!("{title}-{timestamp}"));
                let data = RSSResponse::create(None, Some(x)).to_json().unwrap();

                ProducerMessage {
                    size: None,
                    key: "rss".into(),
                    category: Some(String::from("items")),
                    value: MessageValue::Json(data),
                    tag: Some(tag),
                    timestamp: Some(timestamp as u64),
                }
            })
            .collect();

        Ok(result)
    }

    fn prepare_channel(mut channel: rss::Channel) -> Result<ProducerMessage, RhiaqeyError> {
        debug!("preparing channel {}", channel.title);

        let tag = sha256::digest(format!("{}-{}", channel.title, channel.link));

        trace!("tag calculated {}", tag);

        let timestamp = Self::build_timestamp(if channel.pub_date.is_none() {
            channel.last_build_date.clone().unwrap()
        } else {
            channel.pub_date.clone().unwrap()
        });

        trace!("timestamp calculated {}", timestamp);

        channel.set_items(vec![]);

        match RSSResponse::create(Some(channel), None).to_json() {
            Ok(response) => {
                Ok(ProducerMessage {
                    size: Some(1), // override size
                    key: "rss".into(),
                    category: Some(String::from("channel")),
                    value: MessageValue::Json(response),
                    tag: Some(tag),
                    timestamp: Some(timestamp as u64),
                })
            }
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl Producer<RSSSettings> for RSS {
    fn setup(&mut self, settings: Option<RSSSettings>) -> ProducerMessageReceiver {
        info!("setting up {}", self.kind());

        self.settings = Arc::new(RwLock::new(settings.unwrap_or(RSSSettings::default())));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: RSSSettings) {
        let mut locked_settings = self.settings.write().await;
        *locked_settings = settings.clone();
        debug!("new settings updated {:?}", settings);
    }

    async fn start(&mut self) {
        info!("starting {}", self.kind());

        let sender = self.sender.clone().unwrap();
        let settings = self.settings.clone();

        tokio::task::spawn(async move {
            loop {
                let settings = settings.read().await.clone();
                let interval = settings.interval_in_millis;

                if settings.url.is_some() {
                    match Self::fetch_feed(settings).await {
                        Ok(channel) => {
                            info!("feed downloaded");

                            let items = channel.items.clone();

                            if let Ok(message) = Self::prepare_channel(channel) {
                                // sender.send(message).expect("message failed to sent")
                            }

                            if let Ok(items) = Self::prepare_items(items) {
                                debug!("downloaded {} items", items.len());
                                'tt: for message in items {
                                    sender
                                        .send(message.clone())
                                        .expect("message failed to sent");
                                    // debug!("sending message item {:?}", message);
                                    // break 'tt;
                                }
                            }
                        }
                        Err(err) => warn!("error fetching feed: {}", err),
                    }
                } else {
                    warn!("no initial url found in settings");
                }

                thread::sleep(Duration::from_millis(
                    interval.unwrap_or(default_interval().unwrap()),
                ));
            }
        });
    }

    fn kind(&self) -> String {
        String::from("rss")
    }
}
