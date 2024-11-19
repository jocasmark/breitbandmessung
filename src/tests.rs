use log::debug;
use speedtest_rs::speedtest;
use tokio::task;

use crate::{errors::ServiceError, TestResults};

pub async fn perform_all_tests() -> Result<TestResults, ServiceError> {
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
        debug!("Performing download test to server {:?}", best_server.server);
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
        debug!("Performing upload test to server {:?}", best_server.server);
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
        debug!("Performing ping test to server {:?}", best_server.server);

        Ok::<f64, ServiceError>(best_server.latency.as_secs_f64())
    })
    .await??;

    Ok(result * 1000.0) // Convert to milliseconds
}
