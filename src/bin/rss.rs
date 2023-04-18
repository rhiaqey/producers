use rhiaqey_producers::rss::rss::{RSSSettings, RSS};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<RSS, RSSSettings>().await
}
