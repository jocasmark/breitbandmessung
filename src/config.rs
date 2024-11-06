use std::env;

use log::LevelFilter;

#[derive(Debug, Clone)]
pub struct Config {
    pub check_interval: u64,
    pub mqtt_id: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub log_level: LevelFilter,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            check_interval: env::var("CHECK_INTERVAL")
                .ok()
                .and_then(|val| val.parse::<u64>().ok())
                .unwrap_or(60),
            mqtt_id: env::var("MQTT_ID").unwrap_or_else(|_| "speedtest".to_string()),
            mqtt_host: env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string()),
            mqtt_username: env::var("MQTT_USERNAME").ok(),
            mqtt_password: env::var("MQTT_PASSWORD").ok(),
            mqtt_port: env::var("MQTT_PORT")
                .ok()
                .and_then(|val| val.parse::<u16>().ok())
                .unwrap_or(1883),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()) // default to "info"
                .parse::<LevelFilter>()
                .unwrap_or(LevelFilter::Info),
        }
    }
}
