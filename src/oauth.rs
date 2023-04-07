use crate::error::TempomatError;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

const CLIENT_ID: &str = "3dcfeda8e3aa43748cce54a61e6a3d3a";
const CLIENT_SECRET: &str = "0A339C40026062C9EC06DBB01948B053C46B6888A1D0450E5859F453900077D9"; // Breaking the purpose of OAuth 😎

pub fn generate_access_link(instance: &str, redirect: &str) -> String {
    format!("https://{instance}.atlassian.net/plugins/servlet/ac/io.tempo.jira/oauth-authorize/?client_id={CLIENT_ID}&redirect_uri={redirect}")
}

#[derive(Deserialize)]
pub struct AccessTokensFull {
    pub access_token: String,
    pub expires_in: usize,
    pub token_type: String,
    pub scope: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct GetAccessTokens {
    grant_type: &'static str,
    client_id: &'static str,
    client_secret: &'static str,
    redirect_uri: String,
    code: Option<String>,
    refresh_token: Option<String>,
}

impl GetAccessTokens {
    pub fn get_auth_token(code: String, redirect_uri: String) -> Self {
        Self {
            grant_type: "authorization_code",
            client_id: CLIENT_ID,
            client_secret: CLIENT_SECRET,
            redirect_uri,
            code: Some(code),
            refresh_token: None,
        }
    }

    pub fn refresh_token(refresh_token: String, redirect_uri: String) -> Self {
        Self {
            grant_type: "refresh_token",
            client_id: CLIENT_ID,
            client_secret: CLIENT_SECRET,
            redirect_uri,
            refresh_token: Some(refresh_token),
            code: None,
        }
    }

    pub async fn get_tokens(&self) -> Result<AccessTokensFull, TempomatError> {
        let client = Client::new();
        let response = client
            .post("https://api.tempo.io/oauth/token")
            .form(self)
            .send()
            .await?;

        response.json().await.map_err(Into::into)
    }
}

impl AccessTokensFull {
    /// Revokes the current refresh token
    pub async fn revoke(&self) -> Result<(), TempomatError> {
        #[derive(Serialize)]
        struct RequestTokenRemove {
            token_type_hint: &'static str,
            client_id: &'static str,
            client_secret: &'static str,
            token: String,
        }

        let client = Client::new();
        let response = client
            .post("https://api.tempo.io/oauth/revoke_token/")
            .form(&RequestTokenRemove {
                token_type_hint: "refresh_token",
                client_id: CLIENT_ID,
                client_secret: CLIENT_SECRET,
                token: self.refresh_token.clone(),
            })
            .send()
            .await?;

        if !response.status().is_success() {
            Err(TempomatError::OAuthRevokeFailed(response))?
        }
        Ok(())
    }
}

pub mod server {
    use axum::Server;

    
}
