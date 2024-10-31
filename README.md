
# Speedtest-MQTT

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

### Environment Variables

| Variable           | Default | Description                                                                 |
|--------------------|---------|-----------------------------------------------------------------------------|
| `CHECK_INTERVAL`   | `60`    | Interval (in seconds) between each speed test (download, upload, ping, etc).|

### Example Configuration in `docker-compose.yml`

You can set environment variables in your `docker-compose.yml` file like this:

```yaml
version: '3.8'
services:
  speed_test:
    image: jocas/speedtest-mqtt
    environment:
      - CHECK_INTERVAL=120  # Runs the speed test every 120 seconds
```

## License

This project is licensed under the MIT License.