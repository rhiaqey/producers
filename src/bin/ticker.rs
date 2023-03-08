use rhiaqey_producers::ticker::ticker::Ticker;

#[tokio::main]
async fn main() {
    rhiaqey_producers::exe::run::<Ticker>().await
}
