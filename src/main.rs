use std::env;

use log::{debug, error, info, warn, LevelFilter};
use rumqttc::{Client, MqttOptions};
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
struct TestResults {
    download: f64,
    upload: f64,
    ping: f64,
}

#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let check_interval = env::var("CHECK_INTERVAL")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(60);

    let mqtt_id = env::var("MQTT_ID").unwrap_or("speedtest".to_string());
    let mqtt_host = env::var("MQTT_HOST").unwrap_or("localhost".to_string());
    let mqtt_port = env::var("MQTT_PORT")
        .ok()
        .and_then(|val| val.parse::<u16>().ok())
        .unwrap_or(1883);

    // Channel for sending test results to the MQTT publishing task
    let (result_tx, mut result_rx) = mpsc::channel(1);

    let mut mqttoptions = MqttOptions::new(mqtt_id, mqtt_host, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_clean_session(true);
    let (mqtt_client, mut mqtt_connection) = Client::new(mqttoptions, 10);

    let speed_test_task = task::spawn(async move {
        loop {
            match perform_all_tests().await {
                Ok(results) => {
                    if let Err(err) = result_tx.send(results).await {
                        warn!("Failed to send test results to MQTT: {:?}", err);
                    }
                }
                Err(err) => error!("Speedtest failed: {:?}", err),
            }

            sleep(Duration::from_secs(check_interval)).await;
        }
    });

    // Task to manage MQTT publishing and connection
    let mqtt_publish_task = tokio::spawn(async move {
        while let Some(results) = result_rx.recv().await {
            let payload = format!(
                "Download: {:.2} Mbps, Upload: {:.2} Mbps, Ping: {:.2} ms",
                results.download, results.upload, results.ping
            );

            match mqtt_client.publish(
                "speedtest/results",
                rumqttc::QoS::AtLeastOnce,
                false,
                payload.clone(),
            ) {
                Ok(_) => info!("Published Speedtest result to MQTT: {payload}"),
                Err(err) => error!("MQTT publish error: {:?}", err),
            }
        }
    });

    // Task to handle MQTT connection events
    let mqtt_eventloop_task = tokio::spawn(async move {
        loop {
            match mqtt_connection.eventloop.poll().await {
                Ok(notification) => {
                    debug!("Received MQTT event: {:?}", notification);
                }
                Err(err) => {
                    error!("MQTT connection error: {:?}", err);
                    break;
                }
            }
        }
    });

    let _ = tokio::join!(speed_test_task, mqtt_publish_task, mqtt_eventloop_task);
    Ok(())
}

async fn perform_all_tests() -> Result<TestResults, ServiceError> {
    let download_task = task::spawn(perform_download_test());
    let upload_task = task::spawn(perform_upload_test());
    let ping_task = task::spawn(perform_ping_test());

    let (download, upload, ping) = tokio::join!(download_task, upload_task, ping_task);

    // Flatten and process results using a helper function
    let download = download.map_err(|_| ServiceError::TaskJoinError)??;
    let upload = upload.map_err(|_| ServiceError::TaskJoinError)??;
    let ping = ping.map_err(|_| ServiceError::TaskJoinError)??;

    Ok(TestResults {
        download,
        upload,
        ping,
    })
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
