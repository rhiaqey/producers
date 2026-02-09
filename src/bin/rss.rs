use rhiaqey_producer_rss::rss::{RSS, RSSSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<RSS, RSSSettings>().await
}
