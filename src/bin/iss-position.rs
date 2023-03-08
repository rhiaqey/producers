use rhiaqey_producers::iss::position::ISSPosition;

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<ISSPosition>().await
}
