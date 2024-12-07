use std::path::PathBuf;

use config::{ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub jar_path: PathBuf,
    pub envs_path: PathBuf,
}

impl AppConfig {
    pub fn new(conf_file: impl AsRef<str>) -> Result<Self, ConfigError> {
        let s = config::Config::builder()
            .add_source(File::with_name(conf_file.as_ref()))
            .add_source(
                Environment::with_prefix("LAZYPROP")
                    .try_parsing(true)
                    .separator("_"),
            );

        s.build()?.try_deserialize()
    }
}
