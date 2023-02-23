mod executor;
mod http;

use futures::StreamExt;
use lazy_static::lazy_static;
use log::{debug, trace, warn};
use prometheus::{register_gauge, Gauge};
use rhiaqey_common::env::parse_env;
use rhiaqey_common::settings::parse_settings;
use rhiaqey_sdk::producer::Producer;
use serde::de::DeserializeOwned;

use crate::exe::executor::Executor;
use crate::exe::http::start_http_server;

lazy_static! {
    static ref TOTAL_CHANNELS: Gauge =
        register_gauge!("total_channels", "Total number of active channels.",)
            .expect("cannot create gauge metric for channels");
}

pub async fn run<P: Producer<S> + Default + Send + 'static, S: DeserializeOwned + Default>() {
    env_logger::init();
    let env = parse_env();

    let mut executor = match Executor::setup(env).await {
        Ok(exec) => exec,
        Err(error) => {
            panic!("failed to setup executor: {error}");
        }
    };

    debug!(
        "producer [id={},name={}] is ready",
        executor.get_id(),
        executor.get_name()
    );

    let channels = executor.get_channels().await;
    let port = executor.get_private_port();
    let channel_count = channels.len() as f64;
    executor.set_channels(channels).await;

    let mut plugin = P::default();
    let settings = parse_settings::<S>();
    if settings.is_none() {
        warn!("settings could not be found");
    }

    TOTAL_CHANNELS.set(channel_count);

    let mut publisher_stream = match plugin.setup(settings) {
        Err(error) => {
            panic!("failed to setup producer: {error}");
        }
        Ok(sender) => sender,
    };

    tokio::spawn(async move {
        plugin.start();
    });

    tokio::spawn(async move { start_http_server(port).await });

    let mut pubsub_stream = executor.create_hub_to_publishers_pubsub().await.unwrap();

    loop {
        tokio::select! {
            Some(message) = publisher_stream.recv() => {
                trace!("message received from plugin: {:?}", message);
                executor.publish(message).await;
            },
            Some(pubsub_message) = pubsub_stream.next() => {
                trace!("message received from pubsub");
                if let Ok(message) = pubsub_message {
                    executor.handle_pubsub_message(message).await;
                }
            }
        }
    }
}
