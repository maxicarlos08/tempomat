use super::oauth::TempoAccessTokens;
use crate::{error::TempomatError, jira::types::Myself};
use chrono::{NaiveDate, NaiveDateTime};
use reqwest::Client;
use serde::Serialize;
use tracing::{error, instrument};

#[derive(Debug)]
pub struct TempoApi<'a>(pub &'a TempoAccessTokens);

impl<'a> TempoApi<'a> {
    #[instrument(level = "trace")]
    pub async fn create_worklog(
        &self,
        me: &Myself,
        issue_id: &str,
        description: Option<String>,
        time_spent: usize,
        start: NaiveDateTime,
    ) -> Result<bool, TempomatError> {
        #[derive(Serialize)]
        struct Payload<'a> {
            #[serde(rename = "authorAccountId")]
            author_account_id: &'a str,
            #[serde(rename = "issueId")]
            issue_id: &'a str,
            #[serde(rename = "startDate")]
            start_date: NaiveDate,
            #[serde(rename = "startTime")]
            start_time: String,
            #[serde(rename = "timeSpentSeconds")]
            time_spent: usize,
            description: Option<String>,
        }

        let client = Client::new();
        let response = client
            .post("https://api.tempo.io/4/worklogs")
            .bearer_auth(&self.0.access_token)
            .json(&Payload {
                author_account_id: &me.account_id,
                issue_id,
                time_spent,
                start_date: start.date(),
                start_time: start.time().format("%H:%M:%S").to_string(),
                description,
            })
            .send()
            .await?;

        if response.status().is_success() {
            Ok(true)
        } else {
            error!(
                "Got error when creating worklog: {}",
                response.text().await?
            );
            Ok(false)
        }
    }
}
