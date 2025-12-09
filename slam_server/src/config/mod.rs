use serde::Deserialize;
use std::fs;
use std::path::Path;

const DEFAULT_DB_PATH: &str = "sport.db";
const DEFAULT_SERVER_IP: &str = "127.0.0.1";
const DEFAULT_SERVER_PORT: u16 = 3000;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_ip")]
    pub ip: String,
    #[serde(default = "default_server_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    #[serde(default = "default_db_path")]
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_ai_key")]
    pub key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_security_salt")]
    pub salt: String,
    #[serde(default = "default_security_key")]
    pub key: String,
    #[serde(default = "default_jwt_ttl_seconds")]
    pub jwt_ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub security: SecurityConfig,
}

fn default_db_path() -> String { DEFAULT_DB_PATH.to_string() }
fn default_server_ip() -> String { DEFAULT_SERVER_IP.to_string() }
fn default_server_port() -> u16 { DEFAULT_SERVER_PORT }
fn default_ai_key() -> String { "".to_string() }
fn default_security_salt() -> String { "slam-server-salt".to_string() }
fn default_security_key() -> String { "change-me-key".to_string() }
fn default_jwt_ttl_seconds() -> u64 { 2592000 }

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

impl Default for ServerConfig {
    fn default() -> Self { Self { ip: DEFAULT_SERVER_IP.to_string(), port: DEFAULT_SERVER_PORT } }
}
impl Default for DbConfig {
    fn default() -> Self { Self { path: DEFAULT_DB_PATH.to_string() } }
}
impl Default for AiConfig {
    fn default() -> Self { Self { key: "".to_string() } }
}
impl Default for SecurityConfig {
    fn default() -> Self { Self { salt: default_security_salt(), key: default_security_key(), jwt_ttl_seconds: default_jwt_ttl_seconds() } }
}
