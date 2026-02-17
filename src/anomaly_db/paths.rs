use std::fs;
use std::path::PathBuf;

use crate::config::Config;

#[derive(Debug, Clone)]
pub(crate) struct DbPaths {
    pub(crate) events_dir: PathBuf,
    pub(crate) index_dir: PathBuf,
    pub(crate) meta_dir: PathBuf,
}

pub(crate) fn paths_from_config(config: &Config) -> DbPaths {
    let root = PathBuf::from(&config.anomaly_db.dir);
    DbPaths {
        events_dir: root.join("events"),
        index_dir: root.join("index"),
        meta_dir: root.join("meta"),
    }
}

pub(crate) fn ensure_db_dirs(paths: &DbPaths) -> Result<(), std::io::Error> {
    fs::create_dir_all(&paths.events_dir)?;
    fs::create_dir_all(&paths.index_dir)?;
    fs::create_dir_all(&paths.meta_dir)?;
    Ok(())
}