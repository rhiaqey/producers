use rhiaqey_producer_yahoo::yahoo::{Yahoo, YahooSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<Yahoo, YahooSettings>().await
}
