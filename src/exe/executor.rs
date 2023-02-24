use log::{debug, info, trace, warn};
use rhiaqey_common::env::Env;
use rhiaqey_common::pubsub::{RPCMessage, RPCMessageData};
use rhiaqey_common::redis::connect_and_ping;
use rhiaqey_common::stream::{StreamMessage, StreamMessageDataType};
use rhiaqey_common::topics;
use rhiaqey_sdk::channel::{Channel, ChannelList};
use rhiaqey_sdk::producer::ProducerMessage;
use rustis::client::{Client, PubSubMessage, PubSubStream};
use rustis::commands::{PubSubCommands, StreamCommands, StringCommands, XAddOptions};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct Executor {
    env: Arc<Env>,
    channels: Arc<RwLock<Vec<Channel>>>,
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
        self.env.private_port.unwrap()
    }

    pub async fn set_channels(&mut self, channels: Vec<Channel>) {
        let mut locked_channels = self.channels.write().await;
        *locked_channels = channels;
    }

    pub async fn get_channels(&self) -> Vec<Channel> {
        let channels_key =
            topics::publisher_channels_key(self.env.namespace.clone(), self.env.name.clone());

        let result: String = self
            .redis
            .lock()
            .await
            .as_mut()
            .unwrap()
            .get(channels_key.clone())
            .await
            .unwrap();

        let channel_list: ChannelList =
            serde_json::from_str(result.as_str()).unwrap_or(ChannelList::default());

        debug!(
            "channels from {} retrieved {:?}",
            channels_key, channel_list
        );

        channel_list.channels
    }

    pub async fn setup(config: Env) -> Result<Executor, String> {
        let redis_connection = connect_and_ping(config.redis.clone()).await;
        if redis_connection.is_none() {
            return Err("failed to connect to redis".to_string());
        }

        Ok(Executor {
            env: Arc::from(config),
            channels: Arc::from(RwLock::new(vec![])),
            redis: Arc::new(Mutex::new(redis_connection)),
        })
    }

    async fn handle_rpc_message(&mut self, message: RPCMessage) {
        match message.data {
            RPCMessageData::AssignChannels(channel_list) => {
                info!("received assign channels rpc {:?}", channel_list);
                self.set_channels(channel_list.channels).await;
            }
            _ => {}
        }
    }

    pub async fn handle_pubsub_message(&mut self, message: PubSubMessage) {
        trace!("handle pubsub message");
        if let Ok(data) = serde_json::from_slice::<RPCMessage>(message.payload.as_slice()) {
            trace!("pubsub message contains an RPC message {:?}", data);
            self.handle_rpc_message(data).await;
        }
    }

    pub async fn create_hub_to_publishers_pubsub(&mut self) -> Option<PubSubStream> {
        let client = connect_and_ping(self.env.redis.clone()).await;
        if client.is_none() {
            warn!("failed to connect with ping");
            return None;
        }

        let key = topics::hub_to_publisher_pubsub_topic(
            self.env.namespace.clone(),
            self.env.name.clone(),
        );

        let stream = client.unwrap().subscribe(key.clone()).await.unwrap();

        Some(stream)
    }

    pub async fn publish(&self, message: ProducerMessage) {
        info!("publishing message to the channels");

        let mut stream_msg: StreamMessage = StreamMessage {
            hub_id: None,
            publisher_id: None,
            data_type: StreamMessageDataType::Data as u8,
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

        // if self.is_debug() {
        stream_msg.publisher_id = Some(self.env.id.clone());
        // }

        for channel in self.channels.read().await.iter() {
            stream_msg.channel = channel.name.to_string();
            stream_msg.size = Some(message.size.unwrap_or(channel.size));

            let topic = topics::publishers_to_hub_stream_topic(
                self.env.namespace.clone(),
                channel.name.clone(),
            );

            info!(
                "publishing message channel={}, max_len={}, topic={}, timestamp={:?}",
                channel.name, channel.size, topic, stream_msg.timestamp,
            );

            if let Ok(data) = serde_json::to_string(&stream_msg) {
                let id: String = self
                    .redis
                    .lock()
                    .await
                    .as_mut()
                    .unwrap()
                    .xadd(
                        topic.clone(),
                        "*",
                        [("raw", data.clone())],
                        XAddOptions::default(),
                    )
                    .await
                    .unwrap();
                debug!(
                    "sent message {} to channel {} in topic {}",
                    id, channel.name, topic
                );
            }
        }
    }
}
