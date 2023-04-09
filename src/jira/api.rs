use reqwest::Client;

use super::{
    types::{Issue, JiraIssueKey},
    AtlassianTokens,
};
use crate::{config::Config, error::TempomatError};

pub struct JiraApi<'a>(pub &'a AtlassianTokens, pub &'a Config);

impl<'a> JiraApi<'a> {
    pub async fn get_issue(&self, key: &JiraIssueKey) -> Result<Issue, TempomatError> {
        let client = Client::new();
        let response: Issue = client
            .get(format!(
                "https://{}.atlassian.net/rest/api/3/issue/{}",
                self.1.atlassian_instance,
                key.to_string()
            ))
            .header("Accept", "application/json")
            .basic_auth(&self.0.email, Some(&self.0.token))
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}
