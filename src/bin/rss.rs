use rhiaqey_producer_rss::rss::{RSSSettings, RSS};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<RSS, RSSSettings>().await
}
