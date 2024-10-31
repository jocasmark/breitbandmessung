use log::{error, info, warn, LevelFilter};
use std::fmt::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::sleep;

#[derive(Debug)]
enum TestResult {
    Download(f64), // Download speed in Mbps
    Upload(f64),   // Upload speed in Mbps
}

#[tokio::main]
async fn main() {
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
        }
    }

    // Await task completion (optional, for cleanup)
    let _ = tokio::join!(download_task, upload_task);
}

async fn perform_download_test() -> Result<f64, Error> {
    Ok(50.0)
}
async fn perform_upload_test() -> Result<f64, Error> {
    Ok(10.0)
}
