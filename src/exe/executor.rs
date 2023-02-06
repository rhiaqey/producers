use log::{debug, trace};
use rhiaqey_common::env::Env;
use rhiaqey_common::stream::{StreamMessage, StreamMessageDataType};
use rhiaqey_sdk::channel::ChannelList;
use rhiaqey_sdk::producer::ProducerMessage;
use rustis::client::Client;
use rustis::commands::{ConnectionCommands, PingOptions, StreamCommands, XAddOptions};
use std::sync::Arc;
use tokio::sync::Mutex;
use rhiaqey_common::redis;

pub struct Executor {
    env: Arc<Env>,
    redis: Arc<Mutex<Option<Client>>>,
}

impl Executor {
    pub fn get_id(&self) -> String {
        self.env.id.clone()
    }

    pub fn get_name(&self) -> String {
        self.env.name.clone()
    }

    pub fn get_private_port(&self) -> u16 {
        self.env.private_port
    }

    pub fn get_channels(&self) -> ChannelList {
        self.env.channels.clone()
    }

    pub fn is_debug(&self) -> bool {
        self.env.debug
    }

    pub fn get_channel_topic(&self, channel: String) -> String {
        format!("{}:hub:channels:{channel}:in", self.env.namespace)
    }

    pub async fn setup(config: Env) -> Result<Executor, String> {
        let redis_connection = redis::connect(config.redis.clone()).await;
        let result: String = redis_connection
            .clone()
            .unwrap()
            .ping(PingOptions::default().message("hello"))
            .await
            .unwrap();
        if result != "hello" {
            return Err("ping failed".to_string());
        }

        Ok(Executor {
            env: Arc::from(config),
            redis: Arc::new(Mutex::new(redis_connection)),
        })
    }

    pub async fn publish(&mut self, message: ProducerMessage) {
        trace!("publishing message");

        let mut stream_msg: StreamMessage = StreamMessage {
            hub_id: None,
            publisher_id: None,
            msg_type: StreamMessageDataType::Data as u8,
            channel: "".to_string(),
            tag: message.tag,
            key: message.key,
            value: message.value,
            category: message.category,
            size: None,
            client_ids: None,
            group_ids: None,
            user_ids: None,
            timestamp: message.timestamp,
        };

        if self.is_debug() {
            stream_msg.publisher_id = Some(self.env.id.clone());
        }

        for channel in self.env.channels.channels.iter() {
            stream_msg.channel = channel.name.to_string();
            stream_msg.size = Some(message.size.unwrap_or(channel.size));
            let topic = self.get_channel_topic(channel.name.to_string());

            trace!(
                "publishing message channel={}, max_len={}, topic={}, timestamp={:?}",
                channel.name,
                channel.size,
                topic,
                stream_msg.timestamp,
            );

            if let Ok(data) = rmp_serde::encode::to_vec_named(&stream_msg) {
                let id: String = self
                    .redis
                    .lock()
                    .await
                    .as_mut()
                    .unwrap()
                    .xadd(topic, "*", [("raw", data.clone())], XAddOptions::default())
                    .await
                    .unwrap();
                debug!("sent message {} to channel {}", id, channel.name);
            }
        }
    }
}
