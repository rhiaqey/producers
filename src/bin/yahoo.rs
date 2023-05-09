use rhiaqey_producers::yahoo::yahoo::{Yahoo, YahooSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<Yahoo, YahooSettings>().await
}
