use chrono::Duration;
use clap::Parser;
use colored::Colorize;
use std::{env, fs, path::PathBuf};
use tempomat::{
    args::{CLISubcommand, TempomatCLI},
    config::{APITokens, Config, Saveable},
    dirs,
    error::TempomatError,
    git,
    jira::api::JiraApi,
    tempo::api::TempoApi,
};
use tracing::debug;

#[tokio::main]
async fn main() -> Result<(), TempomatError> {
    tracing_subscriber::fmt::init();

    let args = TempomatCLI::parse();
    let Some(config_root) = args
            .config
            .to_owned()
            .or_else(|| env::var("TEMPOMAT_ROOT").map(PathBuf::from).ok())
            .or_else(|| dirs().map(|d| d.config_local_dir().to_owned()).ok()) else {
            Err(TempomatError::NoProjectDirs)?
        };

    if !config_root.is_dir() {
        fs::create_dir_all(&config_root)?;
    }

    let mut config = Config::try_read(&config_root)?;
    let mut tokens = APITokens::try_read(&config_root)?;

    match args.command {
        CLISubcommand::Log {
            time,
            message,
            issue_id,
        } => {
            let (Some(config), Some(mut tokens)) = (config.take(), tokens.take()) else {
                Err(TempomatError::MissingConfigurations)?
            };

            debug!("Ensuring all tokens are up to date...");
            // Ensure tokens arent outdated
            tokens.refresh_tokens().await?;

            debug!("Parsing issue key");
            let Some(issue_key) = issue_id.or_else(|| git::get_current_branch_key().ok().flatten()) else {
                Err(TempomatError::CouldNotGetJiraIssueKey)?
            };

            let jira_api = JiraApi(&tokens.jira, &config);

            debug!("Getting issue key and user information...");
            let jira_issue = jira_api.get_issue(&issue_key).await?;
            let me = jira_api.get_me().await?;

            let start = chrono::Local::now().naive_local() - Duration::seconds(time.0 as i64);

            debug!("Submitting the worklog");
            let result = TempoApi(&tokens.tempo.tokens)
                .create_worklog(&me, &jira_issue.id, message, time.0, start)
                .await?;

            if result {
                println!(
                    "Successfully logged {} for issue '{}'",
                    time.1.green(),
                    jira_issue.fields.summary.bright_blue()
                );
            } else {
                println!("{}", "Failed to create worklog, check logs".red());
            }
        }
        CLISubcommand::Login { atlassian_instance } => {
            let config = Config { atlassian_instance };
            let access_tokens = APITokens::initialize(&config).await?;

            config.save(&config_root)?;
            access_tokens.save(&config_root)?;
        }
    }

    Ok(())
}
