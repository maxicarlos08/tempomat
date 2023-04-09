use crate::{
    error::TempomatError,
    jira::{self, types::JiraIssueKey},
};
use std::process::Command;

pub fn get_current_branch_key() -> Result<Option<JiraIssueKey>, TempomatError> {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    let text = String::from_utf8(output.stdout)?;

    Ok(jira::parse_issue_key_fuzzy(&text))
}
