use rhiaqey_producers::ticker::ticker::{Ticker, TickerSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<Ticker, TickerSettings>().await
}
