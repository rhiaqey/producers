use fastping_rs::{PingResult, Pinger as LibPinger};
use log::{debug, info, trace, warn};
use rhiaqey_sdk_rs::message::MessageValue;
use rhiaqey_sdk_rs::producer::{
    Producer, ProducerConfig, ProducerMessage, ProducerMessageReceiver,
};
use rhiaqey_sdk_rs::settings::Settings;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;

fn default_interval() -> Option<u64> {
    Some(30_000) // 30 seconds
}

fn default_max_round_trip() -> Option<u64> {
    Some(5000) // 5 seconds
}

fn default_ping_data_packet_size() -> Option<usize> {
    Some(16) // 16 bytes
}

fn default_addresses() -> Option<Vec<String>> {
    Some(vec![
        String::from("urnovl.co"),
        String::from("rhiaqey.com"),
        String::from("8.8.8.8"),
        String::from("1.1.1.1"),
    ])
}

#[derive(Deserialize, Clone, Debug)]
pub struct PingerSettings {
    #[serde(alias = "MaxRoundTrip", default = "default_max_round_trip")]
    pub max_roundtrip: Option<u64>,

    #[serde(
        alias = "PingDataPacketSize",
        default = "default_ping_data_packet_size"
    )]
    pub packet_size: Option<usize>,

    #[serde(alias = "Addresses", default = "default_addresses")]
    pub addresses: Option<Vec<String>>,

    #[serde(alias = "Interval", default = "default_interval")]
    pub interval_in_millis: Option<u64>,
}

impl Default for PingerSettings {
    fn default() -> Self {
        PingerSettings {
            addresses: default_addresses(),
            max_roundtrip: default_max_round_trip(),
            packet_size: default_ping_data_packet_size(),
            interval_in_millis: default_interval(),
        }
    }
}

impl Settings for PingerSettings {
    //
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PingResultBody {
    Idle { addr: String },
    Receive { addr: String, rtt: Duration },
}

#[derive(Debug)]
pub struct Pinger {
    sender: Option<UnboundedSender<ProducerMessage>>,
    settings: Arc<Mutex<PingerSettings>>,
}

impl Default for Pinger {
    fn default() -> Self {
        Self {
            sender: None,
            settings: Default::default(),
        }
    }
}

impl Producer<PingerSettings> for Pinger {
    async fn setup(
        &mut self,
        _config: ProducerConfig,
        settings: Option<PingerSettings>,
    ) -> ProducerMessageReceiver {
        info!("setting up {}", Self::kind());

        self.settings = Arc::new(Mutex::new(settings.unwrap_or(PingerSettings::default())));

        let (sender, receiver) = unbounded_channel::<ProducerMessage>();
        self.sender = Some(sender);

        Ok(receiver)
    }

    async fn set_settings(&mut self, settings: PingerSettings) {
        let mut locked_settings = self.settings.lock().await;
        *locked_settings = settings;
        debug!("new settings updated");
    }

    async fn start(&mut self) {
        info!("starting {}", Self::kind());

        let sender = self.sender.clone().unwrap();
        let settings = self.settings.clone();

        tokio::task::spawn(async move {
            loop {
                let settings = settings.lock().await.clone();
                let interval = settings.interval_in_millis;
                let tag = Some(String::from("pinger"));

                let epoch = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();

                let now = epoch.as_millis();

                let (pinger, results) =
                    match LibPinger::new(settings.max_roundtrip, settings.packet_size) {
                        Ok((pinger, results)) => (pinger, results),
                        Err(e) => {
                            warn!("Error creating pinger: {}", e);
                            tokio::time::sleep(Duration::from_millis(interval.unwrap())).await;
                            continue;
                        }
                    };

                let mut total_addresses = 0;
                let mut total_input_addresses = 0;
                let mut addresses: HashMap<String, HashMap<IpAddr, Option<PingResultBody>>> =
                    HashMap::new();

                settings.addresses.unwrap_or(vec![]).iter().for_each(|x| {
                    trace!("Adding address: {}", x);
                    total_input_addresses += 1;

                    match x.parse::<IpAddr>() {
                        Ok(x_parsed) => {
                            pinger.add_ipaddr(x.as_str());

                            let mut results: HashMap<IpAddr, Option<PingResultBody>> =
                                HashMap::new();
                            results.insert(x_parsed, None);

                            addresses.insert(x.to_string(), results);
                            total_addresses += 1;
                        }
                        Err(err) => {
                            warn!("error parsing ip address[{x}]: {err}");
                            match dns_lookup::lookup_host(x.as_str()) {
                                Ok(result) => {
                                    let mut results: HashMap<IpAddr, Option<PingResultBody>> =
                                        HashMap::new();

                                    result.iter().for_each(|y| {
                                        trace!("Adding address for domain[{x}]: {y}");
                                        pinger.add_ipaddr(y.to_string().as_str());
                                        results.insert(y.to_owned(), None);
                                        total_addresses += 1;
                                    });

                                    addresses.insert(x.to_string(), results);
                                }
                                Err(err) => {
                                    warn!("dns lookup error for {x}: {err}");
                                }
                            }
                        }
                    }
                });

                info!(
                    "found total {} addresses for {} inputs",
                    total_addresses, total_input_addresses
                );

                pinger.ping_once();

                let mut total_results = 0;

                loop {
                    if total_addresses == total_results {
                        info!("received total {} results", total_results);
                        break;
                    }

                    match results.recv() {
                        Ok(result) => match result {
                            PingResult::Idle { addr } => {
                                warn!("Idle address: {}", addr);
                                addresses.iter_mut().for_each(|xx| {
                                    xx.1.iter_mut().for_each(|yy| {
                                        if addr.eq(yy.0) {
                                            *yy.1 = Some(PingResultBody::Idle {
                                                addr: addr.to_string(),
                                            });
                                            total_results += 1;
                                        }
                                    });
                                });
                            }
                            PingResult::Receive { addr, rtt } => {
                                info!("Receive from address: {} in {:?}", addr, rtt);
                                addresses.iter_mut().for_each(|xx| {
                                    xx.1.iter_mut().for_each(|yy| {
                                        if addr.eq(yy.0) {
                                            *yy.1 = Some(PingResultBody::Receive {
                                                addr: addr.to_string(),
                                                rtt,
                                            });
                                            total_results += 1;
                                        }
                                    });
                                });
                            }
                        },
                        Err(err) => {
                            warn!(
                                "Worker threads disconnected before the solution was found: {err}"
                            );
                        }
                    }
                }

                let Ok(value) = serde_json::to_value(addresses) else {
                    warn!("failed to convert addresses to value");
                    continue;
                };

                sender
                    .send(ProducerMessage {
                        tag,
                        key: String::from("pinger"),
                        value: MessageValue::Json(value),
                        category: None, // will be treated as default
                        size: None,
                        timestamp: Option::from(now as u64),
                        user_ids: None,
                        client_ids: None,
                    })
                    .unwrap();

                tokio::time::sleep(Duration::from_millis(interval.unwrap())).await;
            }
        });
    }

    fn schema() -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "UpdateTags": {
                    "type": "boolean",
                    "examples": [ false ],
                },
                "UpdateTimestamp": {
                    "type": "boolean",
                    "examples": [ false ],
                },
                "Interval": {
                    "type": "integer",
                    "examples": [ 15000 ],
                    "minimum": 1000
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
        String::from("pinger")
    }
}
