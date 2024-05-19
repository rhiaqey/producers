use serde::Deserialize;
use std::net::IpAddr;

const PROXY_HUB_LIST_ENDPOINT: &str = "https://plist.ctrader.com/hub/plist";

#[derive(Deserialize, Clone, Debug)]
struct Proxy {
    latency: u16,
    hostname: String,
    ipaddr: IpAddr,
}

#[derive(Deserialize, Clone, Debug)]
struct Hub {
    id: String,
    proxies: Vec<Proxy>,
    plants: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct ProxyHubList {
    date: u64,
    hubs: Vec<Hub>,
}
