use log::{debug, trace};
use rhiaqey_common::env::Env;
use rhiaqey_common::stream::{StreamMessage, StreamMessageDataType};
use rhiaqey_sdk::channel::{Channel, ChannelList};
use rhiaqey_sdk::producer::ProducerMessage;
use rustis::client::{Client, PubSubStream};
use rustis::commands::{ConnectionCommands, PingOptions, PubSubCommands, StreamCommands, StringCommands, XAddOptions};
use std::sync::Arc;
use futures::StreamExt;
use rhiaqey_common::pubsub::{RPCMessage, RPCMessageType};
use tokio::sync::Mutex;
use rhiaqey_common::redis;

pub struct Executor {
    env: Arc<Env>,
    channels: Vec<Channel>,
    redis: Arc<Mutex<Option<Client>>>,
}

impl Executor {
    pub fn get_id(&self) -> String {
        self.env.id.clone()
    }

    pub fn get_name(&self) -> String {
        self.env.name.clone()
    }

    pub fn is_debug(&self) -> bool {
        self.env.debug
    }

    pub fn get_private_port(&self) -> u16 {
        self.env.private_port
    }

    pub fn set_channels(&mut self, channels: Vec<Channel>) {
        self.channels = channels
    }

    pub async fn get_channels(&self) -> Vec<Channel> {
        let key = format!("{}:channels", self.env.namespace);
        let result: String = self.redis.lock().await.as_mut().unwrap().get(key.clone()).await.unwrap();
        let channel_list: ChannelList = serde_json::from_str(result.as_str()).unwrap();
        debug!("channels from {} retrieved {:?}", key, channel_list);
        channel_list.channels
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
            channels: vec!(),
            redis: Arc::new(Mutex::new(redis_connection)),
        })
    }

    async fn handle_rpc_message(&self, message: RPCMessage) {
        match message.data {
            RPCMessageType::AssignChannels(channel_list) => {
                debug!("assign channels {:?}", channel_list);
            }
        }
    }

    pub async fn listen_for_pubsub(&self)  {
        let key = format!("{}:{}:streams:pubsub", self.env.namespace, self.env.name);
        debug!("subscribing to {}", key);
        let mut stream = self.redis.lock().await.as_mut().unwrap().subscribe(key).await.unwrap();
        loop {
            let message = stream.next().await;
            if let Some(result) = message {
                if let Ok(msg) = result {
                    if let Ok(data) = serde_json::from_slice::<RPCMessage>(msg.payload.as_slice()) {
                        self.handle_rpc_message(data).await;
                    }
                }
            }
        }
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

        for channel in self.channels.iter() {
            stream_msg.channel = channel.name.to_string();
            stream_msg.size = Some(message.size.unwrap_or(channel.size));
            // let topic = self.get_channel_topic(channel.name.to_string());

            trace!(
                "publishing message channel={}, max_len={}, timestamp={:?}",
                channel.name,
                channel.size,
                stream_msg.timestamp,
            );

            /*
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
            }*/
        }
    }
}
