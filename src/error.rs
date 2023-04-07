use thiserror::Error;

#[derive(Debug, Error)]
pub enum TempomatError {
    #[error("HTTP error: {0:?}")]
    ReqwestErrror(#[from] reqwest::Error),
    #[error("Failed to revoke OAuth refresh token: {0:?}")]
    OAuthRevokeFailed(reqwest::Response),
    #[error("Failed to join task (this should never happen, please report): {0:?}")]
    JoinError(#[from] tokio::task::JoinError),
}
