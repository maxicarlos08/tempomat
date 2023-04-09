pub mod api;
pub mod types;

use nom::{
    bytes::complete::{tag, take_till, take_while1},
    combinator::{map_res, opt},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::{error::TempomatError, jira::types::JiraIssueKey};
use std::io::{self, Write};

#[derive(Deserialize, Serialize, Debug)]
pub struct AtlassianTokens {
    pub token: String,
    pub email: String,
}

/// Retreives the token from CLI
pub fn get_token() -> Result<AtlassianTokens, TempomatError> {
    fn prompt(prompt: &str) -> Result<String, TempomatError> {
        let mut response = String::new();

        {
            let mut stdout = io::stdout().lock();
            let _ = stdout.write(prompt.as_bytes());
            let _ = stdout.flush();
        }

        io::stdin().read_line(&mut response)?;
        Ok(response.trim().to_string())
    }
    const ATLASSIAN_LINK: &str = "https://id.atlassian.com/manage-profile/security/api-tokens";

    println!("Go to {} and generate a new access token", ATLASSIAN_LINK);

    let _ = io::stdout().flush();
    let _ = open::that(ATLASSIAN_LINK);

    let token = prompt("Paste the token here: ")?;
    let email = prompt("Enter you atlassian email: ")?;

    Ok(AtlassianTokens { token, email })
}

/// Parses a Jira issue id from a string
pub fn parse_issue_key(input: &str) -> IResult<&str, JiraIssueKey> {
    let (input, value) = tuple((
        take_while1(|c: char| c.is_uppercase()),
        tag("-"),
        map_res(take_while1(|c: char| c.is_ascii_digit()), |n: &str| {
            n.parse::<usize>()
        }),
    ))(input)?;

    Ok((input, value.into()))
}

/// Fuzzy version for parse_issue, used for git branches
pub fn parse_issue_key_fuzzy(text: &str) -> Option<JiraIssueKey> {
    fn try_parse_till_issue(input: &str) -> IResult<&str, Option<JiraIssueKey>> {
        let (input, value) =
            tuple((take_till(|c: char| c.is_uppercase()), opt(parse_issue_key)))(input)?;

        Ok((input, value.1))
    }

    let mut input = text;

    while !input.is_empty() {
        match try_parse_till_issue(input) {
            Ok((_, Some(value))) => return Some(value),
            Ok((new_input, None)) => input = new_input,
            Err(_) => return None,
        }
    }

    None
}

#[cfg(test)]
mod test {
    use crate::jira::{parse_issue_key, parse_issue_key_fuzzy, types::JiraIssueKey};

    #[test]
    fn test_issue_key() {
        assert_eq!(
            parse_issue_key("DV-5726").unwrap().1,
            JiraIssueKey {
                board: "DV".to_string(),
                id: 5726
            }
        );
    }

    #[test]
    fn test_issue_key_fuzzy() {
        assert_eq!(
            parse_issue_key_fuzzy("feat/DV-5726").unwrap(),
            JiraIssueKey {
                board: "DV".to_string(),
                id: 5726
            }
        )
    }
}
