pub mod api;
pub mod types;

use serde::{Deserialize, Serialize};

use crate::error::TempomatError;
use std::io::{self, Write};

#[derive(Deserialize, Serialize, Debug)]
pub struct AtlassianToken {
    pub token: String,
    pub email: String,
}

/// Retreives the token from CLI
pub fn get_token() -> Result<AtlassianToken, TempomatError> {
    fn prompt(prompt: &str) -> Result<String, TempomatError> {
        let mut response = String::new();

        {
            let mut stdout = io::stdout().lock();
            let _ = stdout.write(prompt.as_bytes());
            let _ = stdout.flush();
        }

        io::stdin().read_line(&mut response)?;
        Ok(response)
    }
    const ATLASSIAN_LINK: &str = "https://id.atlassian.com/manage-profile/security/api-tokens";

    println!("Go to {} and generate a new access token", ATLASSIAN_LINK);

    let _ = io::stdout().flush();
    let _ = open::that(ATLASSIAN_LINK);

    let token = prompt("Paste the token here: ")?;
    let email = prompt("Enter you atlassian email: ")?;

    Ok(AtlassianToken { token, email })
}
