use crate::error::TempomatError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

const CLIENT_ID: &str = "3dcfeda8e3aa43748cce54a61e6a3d3a";
const CLIENT_SECRET: &str = "0A339C40026062C9EC06DBB01948B053C46B6888A1D0450E5859F453900077D9"; // Breaking the purpose of OAuth 😎

const OAUTH_SERVER_PORT: u16 = 8734;
pub const OAUTH_REDIRECT_URI: &str = "http://127.0.0.1:8734/cb";

pub fn generate_access_link(instance: &str, redirect: &str) -> String {
    format!("https://{instance}.atlassian.net/plugins/servlet/ac/io.tempo.jira/oauth-authorize/?client_id={CLIENT_ID}&redirect_uri={redirect}")
}

/// OAuth tokens for tempo
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct TempoAccessTokens {
    pub access_token: String,
    pub expires_in: usize,
    pub token_type: String,
    pub scope: String,
    pub refresh_token: String,
}

#[derive(Serialize, Debug)]
pub struct GetAccessTokens {
    grant_type: &'static str,
    client_id: &'static str,
    client_secret: &'static str,
    redirect_uri: &'static str,
    code: Option<String>,
    refresh_token: Option<String>,
}

impl GetAccessTokens {
    pub fn get_auth_token(code: String) -> Self {
        Self {
            grant_type: "authorization_code",
            client_id: CLIENT_ID,
            client_secret: CLIENT_SECRET,
            redirect_uri: OAUTH_REDIRECT_URI,
            code: Some(code),
            refresh_token: None,
        }
    }

    pub fn refresh_token(refresh_token: String) -> Self {
        Self {
            grant_type: "refresh_token",
            client_id: CLIENT_ID,
            client_secret: CLIENT_SECRET,
            redirect_uri: OAUTH_REDIRECT_URI,
            refresh_token: Some(refresh_token),
            code: None,
        }
    }

    #[instrument(level = "trace")]
    pub async fn get_tokens(&self) -> Result<TempoAccessTokens, TempomatError> {
        let client = Client::new();
        debug!("Sending request to get OAuth tokens...");
        let response = client
            .post("https://api.tempo.io/oauth/token")
            .form(self)
            .send()
            .await?;
        debug!("Success!");
        response.json().await.map_err(Into::into)
    }
}

impl TempoAccessTokens {
    /// Revokes the current refresh token
    #[instrument(level = "trace")]
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

pub mod actions {
    use super::{
        generate_access_link, server, GetAccessTokens, TempoAccessTokens, OAUTH_REDIRECT_URI,
        OAUTH_SERVER_PORT,
    };
    use crate::{config::Config, error::TempomatError};
    use tracing::instrument;

    /// Create a new oauth token
    #[instrument(level = "trace")]
    pub async fn login(config: &Config) -> Result<TempoAccessTokens, TempomatError> {
        // Start a server in the background
        let server = server::get_code(([127, 0, 0, 1], OAUTH_SERVER_PORT).into());
        let link = generate_access_link(&config.atlassian_instance, OAUTH_REDIRECT_URI);
        // Start the oauth process by opening the initial link in the browser
        let _ = open::that(&link);
        println!("Click \"Accept\" and then \"Onwards\" in your browser tab, if nothing happened click this link: {link}");

        let code = server.await?;

        GetAccessTokens::get_auth_token(code).get_tokens().await
    }

    /// Refresh an existing oauth token
    #[instrument(level = "trace")]
    pub async fn refresh_token(
        tokens: &TempoAccessTokens,
    ) -> Result<TempoAccessTokens, TempomatError> {
        GetAccessTokens::refresh_token(tokens.refresh_token.to_string())
            .get_tokens()
            .await
    }
}

pub mod server {
    use crate::error::TempomatError;
    use axum::{
        extract::{Query, State},
        routing::get,
        Router, Server,
    };
    use serde::Deserialize;
    use std::{net::SocketAddr, sync::Arc};
    use tokio::{
        sync::{oneshot, Mutex, Notify},
        task,
    };
    use tracing::{debug, error, instrument};

    #[instrument(level = "trace", skip_all)]
    pub async fn get_code(host: SocketAddr) -> Result<String, TempomatError> {
        type ServerState = (Arc<Notify>, Arc<Mutex<Option<oneshot::Sender<String>>>>);
        let handle = task::spawn(async move {
            let (tx, rx) = oneshot::channel();
            let notify_done = Arc::new(Notify::new());

            debug!("Starting OAuth web serverr...");
            let server = Server::bind(&host).serve(
                Router::new()
                    .route("/cb", get(handler))
                    .with_state((notify_done.clone(), Arc::new(Mutex::new(Some(tx)))))
                    .into_make_service(),
            );

            #[derive(Deserialize)]
            struct CBQuery {
                code: String,
            }

            #[instrument(level = "trace")]
            async fn handler(
                State((notify, send)): State<ServerState>,
                Query(CBQuery { code }): Query<CBQuery>,
            ) -> &'static str {
                if let Some(send) = send.lock_owned().await.take() {
                    debug!("Got oauth code, sendnig server shutdown signals");
                    let _ = send.send(code);
                    notify.notify_one();

                    "Success! You can now close this tab"
                } else {
                    error!("Failed to get Notifier");
                    "Something went terribly wrong, leave your house immediatly"
                }
            }

            debug!("Waiting for the web server to get a response...");

            let _graceful = server
                .with_graceful_shutdown(async {
                    notify_done.notified().await;
                    debug!("Web server got shutdown signal, shutting down...");
                })
                .await;

            debug!("Web server successfully shutted down");

            rx.await.unwrap()
        });

        Ok(handle.await?)
    }
}
