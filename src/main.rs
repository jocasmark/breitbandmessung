use config::Config;
use errors::ServiceError;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use models::SpeedTestResult;
use rumqttc::{Client, MqttOptions};
use speedtest_rs::speedtest;
use tokio::{
    sync::mpsc,
    task,
    time::{sleep, Duration},
};

mod config;
mod errors;
mod models;

lazy_static! {
    static ref CONFIG: Config = Config::from_env();
}

#[derive(Debug)]
struct TestResults {
    download: f64,
    upload: f64,
    ping: f64,
}

#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    env_logger::builder().filter_level(CONFIG.log_level).init();

    // Channel for sending test results to the MQTT publishing task
    let (result_tx, mut result_rx) = mpsc::channel(1);

    let mut mqttoptions = MqttOptions::new(&CONFIG.mqtt_id, &CONFIG.mqtt_host, CONFIG.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_clean_session(true);
    if let (Some(username), Some(password)) = (&CONFIG.mqtt_username, &CONFIG.mqtt_password) {
        mqttoptions.set_credentials(username, password);
    }
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

            sleep(Duration::from_secs(CONFIG.check_interval)).await;
        }
    });

    // Task to manage MQTT publishing and connection
    let mqtt_publish_task = tokio::spawn(async move {
        while let Some(results) = result_rx.recv().await {
            let speed_test_result =
                SpeedTestResult::new(results.download, results.upload, results.ping);

            let payload = match serde_json::to_string(&speed_test_result) {
                Ok(string) => string,
                Err(err) => {
                    warn!("Failed to serialize SpeedTest result to JSON: {:?}", err);
                    continue;
                }
            };

            match mqtt_client.publish(
                &CONFIG.mqtt_topic,
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
                    std::process::exit(1);
                }
            }
        }
    });

    let _ = tokio::join!(speed_test_task, mqtt_publish_task, mqtt_eventloop_task);
    Ok::<(), ServiceError>(())
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
        let config = speedtest::get_configuration()?;
        let servers = speedtest::get_server_list_with_config(&config)?;
        let best_server = speedtest::get_best_server_based_on_latency(&servers.servers)?;
        let upload_measurement =
            speedtest::test_upload_with_progress_and_config(best_server.server, || {}, &config)?;
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
