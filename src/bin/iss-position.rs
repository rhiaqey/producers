use rhiaqey_producers::iss::position::{ISSPosition, ISSPositionSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run_async::<ISSPosition, ISSPositionSettings>().await
}
