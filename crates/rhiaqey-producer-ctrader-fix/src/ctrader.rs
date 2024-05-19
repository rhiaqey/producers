use rhiaqey_sdk_rs::settings::Settings;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct CTraderSettings {
    hostname: String,
    port: usize,
    password: String,
    sender_comp_id: String,
    target_comp_id: String,
    sender_sub_id: String,
}

impl Default for CTraderSettings {
    fn default() -> Self {
        CTraderSettings {
            hostname: String::from(""),
            port: 0,
            password: String::from(""),
            sender_comp_id: String::from("cTrader"),
            target_comp_id: String::from("cServer"),
            sender_sub_id: String::from("QUOTE"),
        }
    }
}

impl Settings for CTraderSettings {
    //
}
