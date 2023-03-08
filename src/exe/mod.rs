mod executor;
mod http;
mod metrics;

use futures::StreamExt;
use log::{debug, info, trace, warn};
use rhiaqey_common::env::parse_env;
use rhiaqey_common::pubsub::RPCMessageData;
use rhiaqey_sdk::producer::Producer;

use crate::exe::executor::Executor;
use crate::exe::http::start_private_http_server;

pub async fn run<P: Producer + Default + Send + 'static>() {
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
    executor.set_channels(channels).await;

    let mut plugin = P::default();
    let settings = executor.get_settings().await;
    if settings.is_none() {
        warn!("settings could not be found");
    } else {
        info!("setting retrieved successfully")
    }

    let mut publisher_stream = match plugin.setup(settings) {
        Err(error) => {
            panic!("failed to setup producer: {error}");
        }
        Ok(sender) => sender,
    };

    tokio::spawn(async move { start_private_http_server(port).await });

    tokio::spawn(async move { plugin.start().await });

    let mut pubsub_stream = executor.create_hub_to_publishers_pubsub().await.unwrap();

    debug!("stream is ready");

    loop {
        tokio::select! {
            Some(message) = publisher_stream.recv() => {
                trace!("message received from plugin: {:?}", message);
                executor.publish(message).await;
            },
            Some(pubsub_message) = pubsub_stream.next() => {
                trace!("message received from pubsub");
                if let Ok(message) = pubsub_message {
                    if let Some(rpc_message) = executor.extract_pubsub_message(message) {
                        match rpc_message.data {
                            RPCMessageData::AssignChannels(channel_list) => {
                                info!("received assign channels rpc {:?}", channel_list);
                                executor.set_channels(channel_list.channels).await;
                            }
                            RPCMessageData::UpdateSettings(value) => {
                                info!("received request to update settings rpc {:?}", value);
                                /*if let Ok(_settings) = value.decode::<S>() {
                                    // TODO: Properly set settings here
                                    panic!("force restart");
                                }*/
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
