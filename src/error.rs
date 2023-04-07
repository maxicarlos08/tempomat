use thiserror::Error;

#[derive(Debug, Error)]
pub enum TempomatError {
    #[error("HTTP error: {0:?}")]
    ReqwestErrror(#[from] reqwest::Error),
    #[error("Failed to revoke OAuth refresh token: {0:?}")]
    OAuthRevokeFailed(reqwest::Response),
}
