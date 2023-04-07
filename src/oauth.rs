use crate::error::TempomatError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::instrument;

const CLIENT_ID: &str = "3dcfeda8e3aa43748cce54a61e6a3d3a";
const CLIENT_SECRET: &str = "0A339C40026062C9EC06DBB01948B053C46B6888A1D0450E5859F453900077D9"; // Breaking the purpose of OAuth 😎

const OAUTH_SERVER_PORT: u16 = 8734;
const OAUTH_REDIRECT_URI: &str = "http://127.0.0.1:8734/cb";

pub fn generate_access_link(instance: &str, redirect: &str) -> String {
    format!("https://{instance}.atlassian.net/plugins/servlet/ac/io.tempo.jira/oauth-authorize/?client_id={CLIENT_ID}&redirect_uri={redirect}")
}

#[derive(Deserialize, Debug)]
pub struct OAuthAccessTokens {
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

    #[instrument(level = "trace")]
    pub async fn get_tokens(&self) -> Result<OAuthAccessTokens, TempomatError> {
        let client = Client::new();
        let response = client
            .post("https://api.tempo.io/oauth/token")
            .form(self)
            .send()
            .await?;

        response.json().await.map_err(Into::into)
    }
}

impl OAuthAccessTokens {
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
    use super::{server::OAuthServer, OAuthAccessTokens, OAUTH_SERVER_PORT};
    use crate::{
        config::Config,
        error::TempomatError,
        oauth::{generate_access_link, GetAccessTokens, OAUTH_REDIRECT_URI},
    };
    use tracing::instrument;

    /// Create a new oauth token
    #[instrument(level = "trace")]
    pub async fn login(config: &Config) -> Result<OAuthAccessTokens, TempomatError> {
        // Start a server in the background
        let server = OAuthServer::start(([127, 0, 0, 1], OAUTH_SERVER_PORT).into()).await;
        let link = generate_access_link(&config.atlassian_instance, &OAUTH_REDIRECT_URI);
        // Start the oauth process by opening the initial link in the browser
        let _ = open::that(&link);
        println!("Opened in browser, if nothing happened click this link: {link}");

        let code = server.await?;

        GetAccessTokens::get_auth_token(code, OAUTH_REDIRECT_URI.to_string())
            .get_tokens()
            .await
    }

    /// Refresh an existing oauth token
    #[instrument(level = "trace")]
    pub async fn refresh_token(
        tokens: OAuthAccessTokens,
    ) -> Result<OAuthAccessTokens, TempomatError> {
        GetAccessTokens::refresh_token(tokens.refresh_token, OAUTH_REDIRECT_URI.to_string())
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
    use std::{future::Future, net::SocketAddr, sync::Arc};
    use tokio::{
        sync::{oneshot, Mutex, Notify},
        task::{self, JoinHandle},
    };
    use tracing::{error, instrument};

    pub struct OAuthServer {
        handle: JoinHandle<String>,
    }

    impl OAuthServer {
        #[instrument(level = "trace", skip_all)]
        pub async fn start(host: SocketAddr) -> Self {
            let handle = task::spawn(async move {
                let (tx, rx) = oneshot::channel();
                let notify_done = Arc::new(Notify::new());

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
                    State((notify, send)): State<(
                        Arc<Notify>,
                        Arc<Mutex<Option<oneshot::Sender<String>>>>,
                    )>,
                    Query(CBQuery { code }): Query<CBQuery>,
                ) -> &'static str {
                    if let Some(send) = send.lock_owned().await.take() {
                        let _ = send.send(code);
                        notify.notify_one();

                        "Success! You can now clase this"
                    } else {
                        error!("Failed to get Notifier");
                        "Something went terribly wrong, leave your house immediatly"
                    }
                }

                let _graceful = server
                    .with_graceful_shutdown(async {
                        notify_done.notified().await;
                    })
                    .await;

                rx.await.unwrap()
            });

            Self { handle }
        }
    }

    impl Future for OAuthServer {
        type Output = Result<String, TempomatError>;

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            <JoinHandle<String> as Future>::poll(std::pin::Pin::new(&mut self.handle), cx)
                .map_err(Into::into)
        }
    }
}
