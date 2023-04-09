use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TempomatError {
    #[error("HTTP error: {0:?}")]
    ReqwestErrror(#[from] reqwest::Error),
    #[error("Failed to revoke OAuth refresh token: {0:?}")]
    OAuthRevokeFailed(reqwest::Response),
    #[error("Failed to join task (this should never happen, please report): {0:?}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("I/O error: {0:?}")]
    IOError(#[from] std::io::Error),
    #[error("Missing Tempo acces codes")]
    MissingTempoAccess,
    #[error("Missing Jira access codes")]
    MissingJiraAccess,
    #[error("Could not get project directories")]
    NoProjectDirs,
    #[error("I'm Drunk: {0:?}")]
    RonError(#[from] ron::Error),
    #[error("Failed to parse RON file: {0:?}")]
    RonParseError(#[from] ron::error::SpannedError),
    #[error("Missing configuration or tokens")]
    MissingConfigurations,
    #[error("Could not get Jira issue ID, use -i <issue_key> to set it")]
    CouldNotGetJiraIssueKey,
    #[error("Invalid UTF-8 data")]
    InvalidUtf(#[from] FromUtf8Error),
}
