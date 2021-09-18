#![feature(once_cell, never_type)]

use std::panic;
use tokio::sync::broadcast;
use tracing_subscriber::{filter::LevelFilter, util::SubscriberInitExt, EnvFilter, Layer};

use crate::log::LogChannel;

mod config;
mod log;
mod server;

#[no_mangle]
pub extern "C" fn setup() {
    EnvFilter::new(concat!(env!("CARGO_PKG_NAME"), "=debug"))
        .add_directive(LevelFilter::INFO.into())
        .with_subscriber(tracing_android::layer(env!("CARGO_PKG_NAME")).subscriber())
        .init();
    panic::set_hook(quest_hook::panic_hook(false, true));

    let config = crate::config::read();

    let (logs, _) = broadcast::channel(config.log_buffer_size);
    crate::log::init(logs.clone());
    crate::server::init(&config, LogChannel::new(logs));
}

#[no_mangle]
pub extern "C" fn load() {}
