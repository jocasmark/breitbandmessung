use std::env;

use log::{error, info, warn, LevelFilter};
use speedtest_rs::{error::SpeedTestError, speedtest};
use thiserror::Error;
use tokio::{
    sync::mpsc,
    task::{self, JoinError},
    time::{sleep, Duration},
};

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Server list error: {0}")]
    ServerListError(String),
    #[error("Latency error: {0}")]
    LatencyError(String),
    #[error("Download test error: {0}")]
    DownloadTestError(String),
    #[error("Upload test error: {0}")]
    UploadTestError(String),
    #[error("Task join error")]
    TaskJoinError,
    #[error("SpeedTest error: {0:?}")]
    SpeedTest(SpeedTestError),
}

// Implement conversion from `JoinError` to `ServiceError`
impl From<JoinError> for ServiceError {
    fn from(_: JoinError) -> Self {
        ServiceError::TaskJoinError
    }
}

// Implement conversion from `SpeedTestError` to `ServiceError`
impl From<SpeedTestError> for ServiceError {
    fn from(error: SpeedTestError) -> Self {
        ServiceError::SpeedTest(error)
    }
}

#[derive(Debug)]
enum TestResult {
    Download(f64), // Download speed in Mbps
    Upload(f64),   // Upload speed in Mbps
    Ping(f64),     // Ping in ms
}

#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let check_interval = env::var("CHECK_INTERVAL")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(60);

    // Channel for reporting test results
    let (tx, mut rx) = mpsc::channel(32);

    // Spawn task to repeatedly perform download tests
    let download_tx = tx.clone();
    let download_task = task::spawn(async move {
        loop {
            match perform_download_test().await {
                Ok(speed) => {
                    if let Err(err) = download_tx.send(TestResult::Download(speed)).await {
                        warn!("Failed to send download result: {:?}", err);
                        break;
                    }
                }
                Err(err) => error!("Download test failed: {:?}", err),
            }
            sleep(Duration::from_secs(check_interval)).await;
        }
    });

    // Spawn task to repeatedly perform upload tests
    let upload_tx = tx.clone();
    let upload_task = task::spawn(async move {
        loop {
            match perform_upload_test().await {
                Ok(speed) => {
                    if let Err(err) = upload_tx.send(TestResult::Upload(speed)).await {
                        warn!("Failed to send upload result: {:?}", err);
                        break;
                    }
                }
                Err(err) => warn!("Upload test failed: {:?}", err),
            }
            sleep(Duration::from_secs(check_interval)).await;
        }
    });

    // Spawn task to repeatedly perform ping tests
    let ping_tx = tx.clone();
    let ping_task = task::spawn(async move {
        loop {
            match perform_ping_test().await {
                Ok(ping) => {
                    if let Err(err) = ping_tx.send(TestResult::Ping(ping)).await {
                        warn!("Failed to send ping result: {:?}", err);
                        break;
                    }
                }
                Err(err) => warn!("Ping test failed: {:?}", err),
            }
            sleep(Duration::from_secs(check_interval)).await;
        }
    });

    // Continuously receive and log test results
    while let Some(result) = rx.recv().await {
        match result {
            TestResult::Download(speed) => {
                info!("Download speed: {:.2} Mbps", speed);
            }
            TestResult::Upload(speed) => {
                info!("Upload speed: {:.2} Mbps", speed);
            }
            TestResult::Ping(ping) => {
                info!("Ping: {:.2} ms", ping);
            }
        }
    }

    // Await task completion (optional, for cleanup)
    let _ = tokio::join!(download_task, upload_task, ping_task);

    Ok(())
}

async fn perform_download_test() -> Result<f64, ServiceError> {
    let result = task::spawn_blocking(|| {
        let mut config = speedtest::get_configuration()?;
        let servers = speedtest::get_server_list_with_config(&config)?;
        let best_server = speedtest::get_best_server_based_on_latency(&servers.servers)?;
        let download_measurement = speedtest::test_download_with_progress_and_config(
            best_server.server,
            || {},
            &mut config,
        )?;
        Ok::<f64, ServiceError>(download_measurement.bps_f64() / 1_000_000.0) // Convert to Mbps
    })
    .await??;
    Ok(result)
}

async fn perform_upload_test() -> Result<f64, ServiceError> {
    let result = task::spawn_blocking(|| {
        let mut config = speedtest::get_configuration()?;
        let servers = speedtest::get_server_list_with_config(&config)?;
        let best_server = speedtest::get_best_server_based_on_latency(&servers.servers)?;
        let upload_measurement = speedtest::test_upload_with_progress_and_config(
            best_server.server,
            || {},
            &mut config,
        )?;
        Ok::<f64, ServiceError>(upload_measurement.bps_f64() / 1_000_000.0) // Convert to Mbps
    })
    .await??;
    Ok(result)
}

async fn perform_ping_test() -> Result<f64, ServiceError> {
    let result = task::spawn_blocking(|| {
        let config = speedtest::get_configuration().map_err(ServiceError::from)?;
        let servers =
            speedtest::get_server_list_with_config(&config).map_err(ServiceError::from)?;
        let best_server = speedtest::get_best_server_based_on_latency(&servers.servers)
            .map_err(ServiceError::from)?;

        Ok::<f64, ServiceError>(best_server.latency.as_secs_f64())
    })
    .await??;

    Ok(result * 1000.0) // Convert to milliseconds
}
