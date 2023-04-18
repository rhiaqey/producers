use rhiaqey_producers::ping::pinger::{Pinger, PingerSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<Pinger, PingerSettings>().await
}
