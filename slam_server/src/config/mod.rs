use serde::Deserialize;
use std::fs;
use std::path::Path;

const DEFAULT_SQLITE_DB_PATH: &str = "sport.db";

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_sqlite_db_path")]
    pub sqlite_db_path: String,
}

fn default_sqlite_db_path() -> String { DEFAULT_SQLITE_DB_PATH.to_string() }

impl AppConfig {
    pub fn new(cfg_path: &str) -> Self {
        let cfg_path = Path::new(cfg_path);
        if cfg_path.exists() {
            if let Ok(file) = fs::File::open(cfg_path) {
                if let Ok(cfg) = serde_yaml::from_reader::<_, AppConfig>(file) {
                    return cfg;
                }
            }
        }
        serde_yaml::from_str::<AppConfig>("{}").expect("AppConfig default deserialization failed")
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new("config/app.yml")
    }
}