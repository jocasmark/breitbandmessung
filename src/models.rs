use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
pub struct SpeedTestResult {
    pub download: f64,
    pub upload: f64,
    pub ping: f64,
    pub timestamp: DateTime<Utc>,
}

impl SpeedTestResult {
    pub fn new(download: f64, upload: f64, ping: f64) -> Self {
        Self {
            download,
            upload,
            ping,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MqttMessage {
    pub name: String,
    pub state_topic: String,
    pub json_attributes_topic: Option<String>,
    pub unit_of_measurement: Option<String>,
    pub value_template: Option<String>,
    pub payload: String,
}

impl From<SpeedTestResult> for MqttMessage {
    fn from(result: SpeedTestResult) -> Self {
        MqttMessage {
            name: "Speedtest Results".to_string(),
            state_topic: "homeassistant/sensor/speedtest/state".to_string(),
            json_attributes_topic: Some("homeassistant/sensor/speedtest/attributes".to_string()),
            unit_of_measurement: None,
            value_template: Some("{{ value_json.status }}".to_string()),
            payload: json!({
                "status": "ok",
                "download": result.download,
                "upload": result.upload,
                "ping": result.ping
            })
            .to_string(),
        }
    }
}
