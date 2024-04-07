use rhiaqey_producer_pinger::pinger::{Pinger, PingerSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<Pinger, PingerSettings>().await
}
