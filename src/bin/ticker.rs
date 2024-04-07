use rhiaqey_producer_ticker::ticker::{Ticker, TickerSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<Ticker, TickerSettings>().await
}
