use speedtest_rs::error::SpeedTestError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum ServiceError {
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
