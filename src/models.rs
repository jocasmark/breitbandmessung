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
