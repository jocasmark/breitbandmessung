use config::Config;
use errors::ServiceError;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use models::{MqttMessage, SpeedTestResult};
use mqtt::{initialize_mqtt, publish_discovery_message};
use tests::perform_all_tests;
use tokio::{
    sync::mpsc,
    task,
    time::{sleep, Duration},
};

mod config;
mod errors;
mod models;
mod mqtt;
mod tests;

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

    let (mqtt_client, mut mqtt_connection) = initialize_mqtt(&CONFIG).await?;

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
        // Publish discovery message for Home Assistant auto-discovery
        if let Err(err) = publish_discovery_message(&mqtt_client).await {
            error!("MQTT disovery message publish error: {:?}", err);
        };

        while let Some(results) = result_rx.recv().await {
            let speed_test_messages: Vec<MqttMessage> =
                SpeedTestResult::new(results.download, results.upload, results.ping).into();

            // Publish each message individually
            // Publish only the numeric values to each `state_topic`
            for message in speed_test_messages {
                let payload = message.value_template.to_string(); // Send only the numeric value as a string

                match mqtt_client.publish(
                    &message.state_topic,
                    rumqttc::QoS::AtLeastOnce,
                    false,
                    payload,
                ) {
                    Ok(_) => info!(
                        "Published Speedtest result to MQTT: '{} {}' to topic '{}'",
                        message.value_template, message.unit_of_measurement, message.state_topic
                    ),
                    Err(err) => error!("MQTT publish error: {:?}", err),
                }
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
