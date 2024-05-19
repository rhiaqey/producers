use rhiaqey_sdk_rs::settings::Settings;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct CTraderSettings {
    ctrader_plist_hub_id: String,
    ctrader_plist_hub_plant: String,
}

impl Default for CTraderSettings {
    fn default() -> Self {
        CTraderSettings {
            ctrader_plist_hub_id: String::from("hub_live"),
            ctrader_plist_hub_plant: String::from("ctrader"),
        }
    }
}

impl Settings for CTraderSettings {
    //
}
