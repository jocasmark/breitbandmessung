
# SpeedTest-MQTT

This application measures internet speed (download, upload, and ping) and logs the results periodically. 
It uses the `speedtest-rs` library for conducting the speed tests and sends the results over MQTT.

## Features

- **Download Speed** in Mbps
- **Upload Speed** in Mbps
- **Ping** in ms

The application logs the speed test results every 60 seconds and handles errors gracefully with custom error handling.

## Getting Started

### Prerequisites

- Rust (for building the application)
- Docker and Docker Compose (for containerized deployment)

### Building and Running Locally

1. Clone the repository:
   ```bash
   git clone https://github.com/jocasmark/speedtest_mqtt.git
   cd speedtest-mqtt
   ```

2. Build and run the application:
   ```bash
   cargo build --release
   ./target/release/speedtest_mqtt
   ```

### Docker Deployment

The application is Docker-ready for cross-platform builds and optimized for a `scratch` base image. Ensure you have `musl` support for static linking and OpenSSL dependencies correctly configured.

1. **Build the Docker Image**:
   ```bash
   docker build -t jocas/speedtest-mqtt .
   ```

2. **Run with Docker Compose**:
   ```bash
   docker-compose up
   ```

> **Note:** For `scratch` builds, ensure `musl-gcc` is installed to enable static linking. Cross-compilation for multiple architectures can be set up with Docker’s multi-platform build support.

## Usage

The service logs results every minute for:
- **Download speed** in Mbps
- **Upload speed** in Mbps
- **Ping** in milliseconds

Example log output:
```plaintext
[INFO  speedtest_mqtt] Download speed: 50.23 Mbps
[INFO  speedtest_mqtt] Upload speed: 10.75 Mbps
[INFO  speedtest_mqtt] Ping: 27.45 ms
```


## Configuration

The `speedtest-mqtt` service can be configured using environment variables. Below are the available configuration options:

## Configuration

The application can be configured through the following environment variables:

# SpeedTest MQTT

A Rust-based service that periodically performs internet speed tests and publishes the results (download, upload, and ping) to an MQTT broker. This service is useful for monitoring internet connection speeds and latency over time and can be integrated into other IoT or data monitoring systems via MQTT.

## Configuration

The application can be configured through the following environment variables:

### Environment Variables

| Variable         | Default             | Description                                                                |
|------------------|---------------------|----------------------------------------------------------------------------|
| `CHECK_INTERVAL` | `60`                | Interval (in seconds) between each speed test (download, upload, ping).    |
| `MQTT_ID`        | `speedtest`         | The unique identifier for the MQTT client.                                |
| `MQTT_TOPIC`     | `speedtest/results` | The topic to publish to MQTT.                                             |
| `MQTT_HOST`      | `localhost`         | The hostname or IP address of the MQTT broker.                            |
| `MQTT_PORT`      | `1883`              | The port on which the MQTT broker is running.                             |
| `LOG_LEVEL`      | `info`              | The log level for the application (`trace`, `debug`, `info`, `warn`, `error`). Adjusts the verbosity of log output for monitoring or debugging purposes. |

### Example Configuration

To set environment variables, you can use a `.env` file or set them directly in your deployment or Docker Compose configuration.

#### Example .env file

```plaintext
CHECK_INTERVAL=120
MQTT_ID=speedtest
MQTT_TOPIC=speedtest/results
MQTT_HOST=broker.example.com
MQTT_PORT=1883
LOG_LEVEL=info
```

This setup will run speed tests every 120 seconds, connect to an MQTT broker at `broker.example.com` on port `1883`, publish to topic `speedtest/results`, and produce log output at the `info` level.

### Example Configuration in `docker-compose.yml`

You can set environment variables in your `docker-compose.yml` file like this:

```yaml
version: '3.8'
services:
  speed_test:
    image: jocas/speedtest-mqtt
    environment:
      - CHECK_INTERVAL=120             # Runs the speed test every 120 seconds
      - MQTT_ID=speedtest              # Sets the MQTT client ID
      - MQTT_TOPIC=speedtest/resulst   # Sets the MQTT topic to be published to
      - MQTT_HOST=broker.example.com   # The MQTT broker's hostname or IP address
      - MQTT_PORT=1883                 # The MQTT broker's port
      - LOG_LEVEL=info                 # Sets the logging level (e.g., info, debug, warn)
```

## License

This project is licensed under the MIT License.