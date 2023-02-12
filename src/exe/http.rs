use axum::{http::StatusCode, response::IntoResponse};
use axum::Router;
use axum::routing::get;
use log::trace;
use prometheus::{Encoder, TextEncoder};

async fn get_ready() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    let mf = prometheus::gather();
    encoder.encode(&mf, &mut buffer).unwrap();
    (StatusCode::OK, [(
        hyper::header::CONTENT_TYPE, encoder.format_type().to_string()
    )], buffer.into_response())
}

async fn get_version() -> &'static str {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    VERSION
}

pub async fn start_http_server(port: u16) -> hyper::Result<()> {
    let app = Router::new()
        .route("/alive", get(get_ready))
        .route("/ready", get(get_ready))
        .route("/metrics", get(get_metrics))
        .route("/version", get(get_version));

    trace!("running http server @ 0.0.0.0:{}", port);

    axum::Server::bind(&format!("0.0.0.0:{}", port).parse().unwrap())
        .serve(app.into_make_service())
        .await
}