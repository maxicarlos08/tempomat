use crate::{
    error::TempomatError,
    jira,
    tempo::oauth::{actions, TempoAccessTokens},
};
use chrono::{Duration, NaiveDateTime, Utc};
use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tracing::debug;

const AUTH_FILENAME: &str = "auth.ron";
const CONFIG_FILENAME: &str = "config.ron";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub atlassian_instance: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TempoAccessMetadata {
    /// The last time the tempo access token was refreshed
    pub last_refresh: NaiveDateTime,
    /// The OAuth access tokens
    pub tokens: TempoAccessTokens,
}

#[derive(Default, Deserialize, Serialize)]
pub struct APITokens {
    /// Jira only has one access token
    pub jira: Option<String>,
    /// OAuth tokens for tempo
    pub tempo: Option<TempoAccessMetadata>,
}

impl APITokens {
    pub async fn initialize(config: &Config) -> Result<Self, TempomatError> {
        // Not using Result::ok() here since we want the process to fail if something went wrong
        let tempo = Some(actions::login(config).await?.into());
        let jira = Some(jira::get_token()?);

        Ok(Self { tempo, jira })
    }

    /// Refreshes tokens if necesarry, returns true if the token was refreshed
    pub async fn refresh_tokens(&mut self) -> Result<bool, TempomatError> {
        let Some(ref tempo) = self.tempo else {Err(TempomatError::MissingTempoAccess)?};

        if (Utc::now().naive_utc() - tempo.last_refresh)
            > Duration::seconds(tempo.tokens.expires_in as i64)
        {
            debug!("Token expired, getting new tokens...");
            let tokens = actions::refresh_token(&tempo.tokens).await?;
            self.tempo = Some(tokens.into());
            Ok(true)
        } else {
            debug!("Tokens not expired, not doing anything");
            Ok(false)
        }
    }
}

impl Saveable for APITokens {
    fn path(dirs: &ProjectDirs) -> PathBuf {
        dirs.config_local_dir().join(AUTH_FILENAME)
    }
}

impl Saveable for Config {
    fn path(dirs: &ProjectDirs) -> PathBuf {
        dirs.config_local_dir().join(CONFIG_FILENAME)
    }
}

trait Saveable: Serialize + DeserializeOwned {
    fn path(dirs: &ProjectDirs) -> PathBuf;

    fn save(&self, dirs: &ProjectDirs) -> Result<(), TempomatError> {
        let path = Self::path(dirs);

        fs::write(path, ron::to_string(self)?)?;

        Ok(())
    }

    fn try_read(dirs: &ProjectDirs) -> Result<Self, TempomatError> {
        let path = Self::path(dirs);
        let config = fs::read_to_string(path)?;
        let config = ron::from_str(&config)?;

        Ok(config)
    }
}

impl From<TempoAccessTokens> for TempoAccessMetadata {
    fn from(tokens: TempoAccessTokens) -> Self {
        Self {
            tokens,
            last_refresh: Utc::now().naive_utc(),
        }
    }
}
