use crate::error::TempomatError;
use std::io::{self, Write};

/// Retreives the token from CLI
pub fn get_token() -> Result<String, TempomatError> {
    const ATLASSIAN_LINK: &str = "https://id.atlassian.com/manage-profile/security/api-tokens";
    let mut token = String::new();

    print!(
        r#"Go to {} and generate a new access token.
When done insert here: "#,
        ATLASSIAN_LINK
    );

    let _ = io::stdout().flush();
    let _ = open::that(ATLASSIAN_LINK);

    io::stdin().read_line(&mut token)?;

    Ok(token)
}
