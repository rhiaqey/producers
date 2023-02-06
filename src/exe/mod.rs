mod executor;

use axum::http::StatusCode;
use axum::response::{IntoResponse};
use axum::Router;
use axum::routing::get;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use prometheus::{register_gauge, Encoder, Gauge, TextEncoder};
use rhiaqey_common::env::parse_env;
use rhiaqey_common::settings::parse_settings;
use rhiaqey_sdk::producer::Producer;
use serde::de::DeserializeOwned;

use crate::exe::executor::Executor;

static mut READY: bool = false;

lazy_static! {
    static ref TOTAL_CHANNELS: Gauge =
        register_gauge!("total_channels", "Total number of active channels.",)
            .expect("cannot create gauge metric for channels");
}

async fn get_ready() -> impl IntoResponse {
    let ready = unsafe { READY };
    if ready {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}

async fn get_metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    let mf = prometheus::gather();
    encoder.encode(&mf, &mut buffer).unwrap();
    buffer.into_response()
}

async fn get_version() -> &'static str {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    VERSION
}

pub async fn run<P: Producer<S> + Default + Send + 'static, S: DeserializeOwned + Default>() {
    env_logger::init();
    let env = parse_env();

    let mut executor = match Executor::setup(env).await {
        Ok(exec) => exec,
        Err(error) => {
            panic!("failed to setup executor: {error}");
        }
    };

    debug!(
        "producer [id={},name={},debug={}] is ready",
        executor.get_id(),
        executor.get_name(),
        executor.is_debug()
    );

    let port = executor.get_private_port();
    let channels = executor.get_channels();

    let mut plugin = P::default();
    let settings = parse_settings::<S>();
    if settings.is_none() {
        warn!("settings could not be found");
    }

    TOTAL_CHANNELS.set(channels.len() as f64);

    let mut rx = match plugin.setup(settings) {
        Err(error) => {
            panic!("failed to setup producer: {error}");
        }
        Ok(sender) => sender,
    };

    tokio::spawn(async move {
        debug!("start receiving messages");
        loop {
            if let Some(message) = rx.recv().await {
                debug!("message about to send");
                executor.publish(message).await;
            }
        }
    });

    tokio::spawn(async move {
        // create router
        let app = Router::new()
            .route("/alive", get(get_ready))
            .route("/ready", get(get_ready))
            .route("/metrics", get(get_metrics))
            .route("/version", get(get_version));

        // run it with hyper on localhost:3000
        axum::Server::bind(&format!("0.0.0.0:{}", port).parse().unwrap())
            .serve(app.into_make_service())
            .await
    });

    info!("running producer {}", P::kind());

    unsafe {
        READY = true;
    }

    plugin.start()
}
