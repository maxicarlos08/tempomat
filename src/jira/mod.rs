use crate::error::TempomatError;
use std::io;

/// Retreives the token from CLI
pub fn get_token() -> Result<String, TempomatError> {
    let mut token = String::new();

    print!(
        r#"Go to https://id.atlassian.com/manage-profile/security/api-tokens and generate a new access token.
When done insert here: "#
    );

    io::stdin().read_line(&mut token)?;

    println!();

    Ok(token)
}
