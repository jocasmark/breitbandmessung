use anyhow::Error;
use env_logger;
use log::{error, info, warn, LevelFilter};
use speedtest_rs::speedtest::{self, SpeedTestResult};
use tokio::{
    sync::mpsc,
    task,
    time::{sleep, Duration},
};

#[derive(Debug)]
enum TestResult {
    Download(f64), // Download speed in Mbps
    Upload(f64),   // Upload speed in Mbps
    Ping(u32),     // Ping in ms
    Jitter(f64),   // Jitter in ms
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the logger
    env_logger::builder().filter_level(LevelFilter::Info).init();

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
            sleep(Duration::from_secs(60)).await;
        }
    });

    // Spawn task to repeatedly perform upload tests
    let upload_task = task::spawn(async move {
        loop {
            match perform_upload_test().await {
                Ok(speed) => {
                    if let Err(err) = tx.send(TestResult::Upload(speed)).await {
                        warn!("Failed to send upload result: {:?}", err);
                        break;
                    }
                }
                Err(err) => warn!("Upload test failed: {:?}", err),
            }
            sleep(Duration::from_secs(60)).await;
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
            _ => continue,
        }
    }

    // Await task completion (optional, for cleanup)
    let _ = tokio::join!(download_task, upload_task);

    Ok(())
}

async fn perform_download_test() -> Result<f64, Error> {
    let result = task::spawn_blocking(|| {
        let mut config = speedtest::get_configuration().unwrap();
        let servers = speedtest::get_server_list_with_config(&config).unwrap();
        let best_server = speedtest::get_best_server_based_on_latency(&servers.servers).unwrap();
        let download_measurement = speedtest::test_download_with_progress_and_config(
            best_server.server,
            || {},
            &mut config,
        )
        .unwrap();
        Ok::<_, Error>(download_measurement.bps_f64() / 1_000_000.0) // Convert to Mbps
    })
    .await??;
    Ok(result)
}

async fn perform_upload_test() -> Result<f64, Error> {
    let result = task::spawn_blocking(|| {
        let mut config = speedtest::get_configuration().unwrap();
        let servers = speedtest::get_server_list_with_config(&config).unwrap();
        let best_server = speedtest::get_best_server_based_on_latency(&servers.servers).unwrap();
        let upload_measurement =
            speedtest::test_upload_with_progress_and_config(best_server.server, || {}, &mut config)
                .unwrap();
        Ok::<_, Error>(upload_measurement.bps_f64() / 1_000_000.0) // Convert to Mbps
    })
    .await??;
    Ok(result)
}

// // Placeholder function for ping test
// async fn perform_ping_test() -> Result<u32, Error> {
//     let options = SpeedTestOptions::default();
//     let speed_test = SpeedTest::new(options);
//     let latency = speed_test.perform_ping()?;
//     Ok(latency)
// }

// // Calculate jitter as the mean absolute deviation from the mean ping time
// fn calculate_jitter(pings: &[u32]) -> f64 {
//     let mean_ping = pings.iter().sum::<u32>() as f64 / pings.len() as f64;
//     pings.iter()
//         .map(|&ping| (ping as f64 - mean_ping).abs())
//         .sum::<f64>() / pings.len() as f64
// }
