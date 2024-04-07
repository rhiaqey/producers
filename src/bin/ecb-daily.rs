use rhiaqey_producer_ecb::daily::{ECBDaily, ECBDailySettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<ECBDaily, ECBDailySettings>().await
}
