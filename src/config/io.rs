use std::path::Path;

use super::{schema::Config, validate::ConfigError};

pub fn load_config(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let path = path.as_ref();
    let path_str = path.display().to_string();
    let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path_str.clone(),
        source,
    })?;
    let config: Config = toml::from_str(&raw).map_err(|source| ConfigError::Parse {
        path: path_str,
        source,
    })?;
    config.validate()?;
    Ok(config)
}
