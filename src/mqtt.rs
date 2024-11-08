use crate::{config::Config, errors::ServiceError};
use log::{error, info, warn};
use rumqttc::{Client, MqttOptions, QoS};
use serde_json::json;
use std::time::Duration;

pub async fn initialize_mqtt(
    config: &Config,
) -> Result<(Client, rumqttc::Connection), rumqttc::ClientError> {
    let mut mqtt_options = MqttOptions::new(&config.mqtt_id, &config.mqtt_host, config.mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_clean_session(true);
    if let (Some(username), Some(password)) = (&config.mqtt_username, &config.mqtt_password) {
        mqtt_options.set_credentials(username, password);
    }
    Ok(Client::new(mqtt_options, 10))
}

pub async fn publish_discovery_message(client: &Client) -> Result<(), ServiceError> {
    let discovery_messages = vec![
        ("download", "Mbit/s", "data_rate"),
        ("upload", "Mbit/s", "data_rate"),
        ("ping", "ms", "duration"),
    ];

    for (name, unit, device_class) in discovery_messages {
        let config_topic = format!("homeassistant/sensor/speedtest/{}/config", name);
        let config_message = json!({
            "name": format!("Speedtest {}", name),
            "state_topic": format!("homeassistant/sensor/speedtest/{}", name),
            "unit_of_measurement": unit,
            "device_class": device_class,
            "unique_id": format!("speedtest_{}", name),
            "device": {
                "name": "Speedtest",
                "identifiers": ["speedtest_device"]
            }
        });

        for attempt in 1..=3 {
            match client.publish(
                config_topic.clone(),
                QoS::AtLeastOnce,
                true,
                config_message.to_string(),
            ) {
                Ok(_) => {
                    info!("Published MQTT discovery message for '{}'", name);
                    break;
                }
                Err(err) if attempt < 3 => {
                    warn!(
                        "Retrying MQTT publish for '{}' (attempt {}/3): {:?}",
                        name, attempt, err
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(err) => {
                    error!(
                        "Failed to publish MQTT discovery message for '{}': {:?}",
                        name, err
                    );
                    return Err(ServiceError::MqttClientError(err));
                }
            }
        }
    }
    Ok(())
}
