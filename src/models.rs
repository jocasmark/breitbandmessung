use chrono::{DateTime, Utc};
use serde::Serialize;

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
    pub unit_of_measurement: String,
    pub value_template: f64,
}

impl From<SpeedTestResult> for Vec<MqttMessage> {
    fn from(result: SpeedTestResult) -> Self {
        vec![
            MqttMessage {
                name: "Speedtest Download".to_string(),
                state_topic: "homeassistant/sensor/Speedtest/Download".to_string(),
                unit_of_measurement: "Mbps".to_string(),
                value_template: result.download,
            },
            MqttMessage {
                name: "Speedtest Upload".to_string(),
                state_topic: "homeassistant/sensor/Speedtest/Upload".to_string(),
                unit_of_measurement: "Mbps".to_string(),
                value_template: result.upload,
            },
            MqttMessage {
                name: "Speedtest Ping".to_string(),
                state_topic: "homeassistant/sensor/Speedtest/Ping".to_string(),
                unit_of_measurement: "ms".to_string(),
                value_template: result.ping,
            },
        ]
    }
}
