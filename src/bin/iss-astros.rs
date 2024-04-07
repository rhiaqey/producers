use rhiaqey_producer_iss::astros::{ISSAstros, ISSAstrosSettings};

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<ISSAstros, ISSAstrosSettings>().await
}
