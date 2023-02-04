pub mod executor;
mod redis;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
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

async fn private_routes(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/alive") => {
            let ready = unsafe { READY };
            if ready {
                Ok(Response::builder().status(200).body(Body::empty()).unwrap())
            } else {
                Ok(Response::builder().status(400).body(Body::empty()).unwrap())
            }
        }

        (&Method::GET, "/ready") => {
            let ready = unsafe { READY };
            if ready {
                Ok(Response::builder().status(200).body(Body::empty()).unwrap())
            } else {
                Ok(Response::builder().status(400).body(Body::empty()).unwrap())
            }
        }

        (&Method::GET, "/metrics") => {
            let encoder = TextEncoder::new();
            let mut buffer = vec![];
            let mf = prometheus::gather();
            encoder.encode(&mf, &mut buffer).unwrap();
            Ok(Response::builder()
                .header(hyper::header::CONTENT_TYPE, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap())
        }

        (&Method::GET, "/version") => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            Ok(Response::new(Body::from(VERSION)))
        }

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
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
        let addr = ([0, 0, 0, 0], port).into();
        debug!("Listening on http://{}", addr);
        let service =
            make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(private_routes)) });
        let server = Server::bind(&addr).serve(service);
        server.await
    });

    info!("running producer {}", P::kind());

    unsafe {
        READY = true;
    }

    plugin.start()
}
