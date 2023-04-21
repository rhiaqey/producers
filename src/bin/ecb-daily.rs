use rhiaqey_producers::ecb::daily::{ECBDaily, ECBDailySettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<ECBDaily, ECBDailySettings>().await
}
