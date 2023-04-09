use crate::{
    error::TempomatError,
    jira::{self, AtlassianTokens},
    tempo::oauth::{actions as tempo_actions, TempoAccessTokens},
};
use chrono::{Duration, NaiveDateTime, Utc};
use colored::Colorize;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
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

#[derive(Deserialize, Serialize)]
pub struct APITokens {
    /// Jira only has one access token
    pub jira: AtlassianTokens,
    /// OAuth tokens for tempo
    pub tempo: TempoAccessMetadata,
}

impl APITokens {
    pub async fn initialize(config: &Config) -> Result<Self, TempomatError> {
        // Not using Result::ok() here since we want the process to fail if something went wrong
        println!("Getting Tempo tokens...");
        let tempo = tempo_actions::login(config).await?.into();
        println!("Getting Jira tokens...");
        let jira = jira::get_token()?;

        println!("{}", "Successfully got access tokens!".green());

        Ok(Self { tempo, jira })
    }

    /// Refreshes tokens if necesarry, returns true if the token was refreshed
    pub async fn refresh_tokens(&mut self) -> Result<bool, TempomatError> {
        if (Utc::now().naive_utc() - self.tempo.last_refresh)
            > Duration::seconds(self.tempo.tokens.expires_in as i64)
        {
            debug!("Token expired, getting new tokens...");
            let tokens = tempo_actions::refresh_token(&self.tempo.tokens).await?;
            self.tempo = tokens.into();
            Ok(true)
        } else {
            debug!("Tokens not expired, not doing anything");
            Ok(false)
        }
    }
}

impl Saveable for APITokens {
    fn path(root: &Path) -> PathBuf {
        root.join(AUTH_FILENAME)
    }
}

impl Saveable for Config {
    fn path(root: &Path) -> PathBuf {
        root.join(CONFIG_FILENAME)
    }
}

pub trait Saveable: Serialize + DeserializeOwned {
    fn path(root: &Path) -> PathBuf;

    fn save(&self, root: &Path) -> Result<(), TempomatError> {
        let path = Self::path(root);

        fs::write(path, ron::to_string(self)?)?;

        Ok(())
    }

    fn try_read(root: &Path) -> Result<Option<Self>, TempomatError> {
        let path = Self::path(root);
        let Ok(config) = fs::read_to_string(path) else { return Ok(None) };
        let config = ron::from_str(&config)?;

        Ok(Some(config))
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
