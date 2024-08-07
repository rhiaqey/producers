use chrono::DateTime;
use futures::TryFutureExt;
use log::{debug, info, trace, warn};
use reqwest::Response;
use rhiaqey_sdk_rs::message::MessageValue;
use rhiaqey_sdk_rs::producer::{
    Producer, ProducerConfig, ProducerMessage, ProducerMessageReceiver,
};
use rhiaqey_sdk_rs::settings::Settings;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::sync::Arc;
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
struct RSSResponse {
    #[serde(rename = "channel", skip_serializing_if = "Option::is_none")]
    pub channel: Option<rss::Channel>,

    #[serde(rename = "item", skip_serializing_if = "Option::is_none")]
    pub item: Option<rss::Item>,
}

impl RSSResponse {
    fn create(channel: Option<rss::Channel>, item: Option<rss::Item>) -> RSSResponse {
        RSSResponse { channel, item }
    }
    fn to_json(&self) -> Result<Value, String> {
        serde_json::to_value(self).map_err(|x| x.to_string())
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

impl Settings for RSSSettings {
    //
}

#[derive(Debug)]
pub struct RSS {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<RwLock<RSSSettings>>,
}

impl RSS {
    async fn send_request(settings: RSSSettings) -> Result<Response, String> {
        info!("fetching feed");

        let client = reqwest::Client::new();
        let timeout = settings
            .timeout_in_millis
            .unwrap_or(default_timeout().unwrap());

        if settings.url.is_none() {
            return Err(String::from("url is not configured properly"));
        }

        if let Some(username) = settings.username {
            return client
                .get(settings.url.unwrap())
                .timeout(Duration::from_millis(timeout))
                .basic_auth(username, settings.password)
                .send()
                .map_err(|x| x.to_string())
                .await;
        }

        client
            .get(settings.url.unwrap())
            .timeout(Duration::from_millis(timeout))
            .send()
            .map_err(|x| x.to_string())
            .await
    }

    async fn fetch_feed(settings: RSSSettings) -> Result<rss::Channel, String> {
        info!("downloading rss feed");

        let res = Self::send_request(settings).await?;
        let text = res.text().await.map_err(|x| x.to_string())?;
        let data = text.as_bytes();
        let channel = rss::Channel::read_from(data).map_err(|x| x.to_string())?;
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

    fn prepare_items(mut items: Vec<rss::Item>) -> Result<Vec<ProducerMessage>, String> {
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

                let tag = sha256::digest(format!("{title}-{timestamp}"));
                let data = RSSResponse::create(None, Some(x)).to_json().unwrap();

                ProducerMessage {
                    size: None,
                    key: "rss".into(),
                    category: Some(String::from("items")),
                    value: MessageValue::Json(data),
                    tag: Some(tag),
                    timestamp: Some(timestamp as u64),
                    user_ids: None,
                    client_ids: None,
                }
            })
            .collect();

        Ok(result)
    }

    fn prepare_channel(mut channel: rss::Channel) -> Result<ProducerMessage, String> {
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
                    user_ids: None,
                    client_ids: None,
                })
            }
            Err(err) => Err(err),
        }
    }
}

impl Default for RSS {
    fn default() -> Self {
        Self {
            sender: None,
            settings: Default::default(),
        }
    }
}

impl Producer<RSSSettings> for RSS {
    async fn setup(
        &mut self,
        _config: ProducerConfig,
        settings: Option<RSSSettings>,
    ) -> ProducerMessageReceiver {
        info!("setting up {}", Self::kind());

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
        info!("starting {}", Self::kind());

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
                                debug!("sending message {:?}", message.clone());
                                sender.send(message).expect("message failed to sent");
                            }

                            if let Ok(items) = Self::prepare_items(items) {
                                debug!("downloaded {} items", items.len());
                                for message in items {
                                    sender
                                        .send(message.clone())
                                        .expect("message failed to sent");
                                }
                            }
                        }
                        Err(err) => warn!("error fetching feed: {}", err),
                    }
                } else {
                    warn!("no initial url found in settings");
                }

                tokio::time::sleep(Duration::from_millis(
                    interval.unwrap_or(default_interval().unwrap()),
                ))
                .await;
            }
        });
    }

    fn schema() -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "Url": {
                    "type": "https://financial-feed/rss.xml",
                    "examples": [ "username" ],
                },
                "Username": {
                    "type": "string",
                    "examples": [ "username" ],
                },
                "Password": {
                    "type": "string",
                    "examples": [ "password" ],
                },
                "Interval": {
                    "type": "integer",
                    "examples": [ 15000 ],
                    "minimum": 1000
                },
                "Timeout": {
                    "type": "integer",
                    "examples": [ 15000 ],
                    "minimum": 1000
                }
            },
            "required": [],
            "additionalProperties": false
        })
    }

    fn kind() -> String {
        String::from("rss")
    }
}
