use std::{
    fs::{self, File},
    path::Path,
};

use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub port: u16,
    pub log_buffer_size: usize,
}

#[tracing::instrument(level = "debug")]
pub fn read() -> Config {
    let dir = Path::new("/sdcard/ModData/com.beatgames.beatsaber/Configs");
    if !dir.exists() {
        debug!("creating config directory");
        fs::create_dir_all(dir).unwrap();
    }

    let file = dir.join(concat!(env!("CARGO_PKG_NAME"), ".json"));
    File::open(&file)
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .unwrap_or_else(|| {
            debug!("writing default config");

            let config = Config::default();
            serde_json::to_writer_pretty(File::create(&file).unwrap(), &config).unwrap();
            config
        })
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 2112,
            log_buffer_size: 256,
        }
    }
}
