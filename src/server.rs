use axum::{
    extract::Extension,
    handler::get,
    response::{sse::Event, Sse},
    AddExtensionLayer, Router, Server,
};
use std::{
    env,
    error::Error,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
    thread,
};
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};
use tower_http::cors::CorsLayer;
use tracing::debug;

use crate::{config::Config, log::LogChannel};

#[tracing::instrument(level = "debug", skip(logs))]
pub fn init(config: &Config, logs: LogChannel) {
    let port = config.port;
    debug!("starting server on port {}...", port);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ID: AtomicUsize = AtomicUsize::new(0);
            format!(
                concat!(env!("CARGO_PKG_NAME"), "-server-{}"),
                ID.fetch_add(1, SeqCst)
            )
        })
        .build()
        .unwrap();

    thread::Builder::new()
        .name(concat!(env!("CARGO_PKG_NAME"), "-server").to_string())
        .spawn(move || runtime.block_on(server(port, logs)).unwrap())
        .unwrap();
}

async fn server(port: u16, logs: LogChannel) -> Result<(), impl Error> {
    let web = include!(concat!(env!("OUT_DIR"), "/web.rs"));

    let app = Router::new()
        .nest("/", web)
        .route("/logs", get(get_logs))
        .layer(AddExtensionLayer::new(logs))
        .layer(CorsLayer::new());

    Server::bind(&([0, 0, 0, 0], port).into())
        .serve(app.into_make_service())
        .await
}

async fn get_logs(
    Extension(logs): Extension<LogChannel>,
) -> Sse<impl Stream<Item = Result<Event, !>>> {
    let stream = BroadcastStream::new(logs.rx())
        .filter_map(|msg| {
            msg.ok()
                .and_then(|msg| Event::default().json_data(msg).ok())
        })
        .map(Ok);
    Sse::new(stream)
}
