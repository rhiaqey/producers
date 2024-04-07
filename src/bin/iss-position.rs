use rhiaqey_producer_iss::position::{ISSPosition, ISSPositionSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<ISSPosition, ISSPositionSettings>().await
}
