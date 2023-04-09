use chrono::{Duration, Local};
use clap::Parser;
use colored::Colorize;
use std::{env, fs, path::PathBuf};
use tempomat::{
    args::{CLISubcommand, TempomatCLI},
    config::{APITokens, Config, Saveable},
    dirs,
    error::TempomatError,
    git,
    jira::{
        api::JiraApi,
        types::{Issue, JiraIssueKey},
    },
    tempo::api::TempoApi,
    timers::TempoTimers,
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

    let now = Local::now().naive_local();

    let get_issue = |issue: Option<JiraIssueKey>| {
        issue
            .or_else(|| git::get_current_branch_key().ok().flatten())
            .ok_or(TempomatError::CouldNotGetJiraIssueKey)
    };

    let requires_auth = |issue: Option<JiraIssueKey>| async move {
        let (Some(config), Some(mut tokens)) = (config.take(), tokens.take()) else {
                Err(TempomatError::MissingConfigurations)?
            };

        debug!("Ensuring all tokens are up to date...");
        // Ensure tokens arent outdated
        tokens.refresh_tokens().await?;

        debug!("Parsing issue key");
        let issue_key = get_issue(issue)?;

        let jira_api = JiraApi(&tokens.jira, &config);

        debug!("Getting issue key and user information...");
        let jira_issue = jira_api.get_issue(&issue_key).await?;
        let me = jira_api.get_me().await?;

        Result::<_, TempomatError>::Ok((jira_issue, me, tokens, issue_key))
    };

    fn show_worklog_result(result: bool, time: &str, issue: &Issue) {
        if result {
            println!(
                "Successfully logged {} for issue '{}'",
                time.green(),
                issue.fields.summary.bright_blue()
            );
        } else {
            println!("{}", "Failed to create worklog, check logs".red());
        }
    }

    match args.command {
        CLISubcommand::Log {
            time,
            description,
            issue,
        } => {
            let (jira_issue, me, tokens, _) = requires_auth(issue).await?;
            let start = now - Duration::seconds(time.0 as i64);

            debug!("Submitting the worklog");
            let result = TempoApi(&tokens.tempo.tokens)
                .create_worklog(&me, &jira_issue.id, description, time.0, start)
                .await?;

            show_worklog_result(result, &time.1, &jira_issue);
        }
        CLISubcommand::Login { atlassian_instance } => {
            let config = Config { atlassian_instance };
            let access_tokens = APITokens::initialize(&config).await?;

            config.save(&config_root)?;
            access_tokens.save(&config_root)?;
        }
        CLISubcommand::Start { issue } => {
            let issue = get_issue(issue)?;
            let mut timers = if let Some(timers) = TempoTimers::try_read(&config_root)? {
                timers
            } else {
                Default::default()
            };

            timers.0.insert(issue.to_string(), now.to_owned());
            timers.save(&config_root)?;

            println!(
                "Started timer on {} for issue {}",
                now.to_string().bright_yellow(),
                issue.to_string().blue()
            );
        }
        CLISubcommand::Stop {
            no_submit,
            description,
            issue,
        } => {
            let timers =
                TempoTimers::try_read(&config_root)?.ok_or(TempomatError::MissingConfigurations)?;
            let (jira_issue, me, tokens, issue) = requires_auth(issue).await?;
            let Some(start) = timers.0.get(&issue.to_string()) else {
                Err(TempomatError::TimerInvalid)?
            };

            if !no_submit {
                let til_now = (now - start.to_owned())
                    .num_seconds()
                    .try_into()
                    .map_err(|_| TempomatError::NegativeTime)?;
                let result = TempoApi(&tokens.tempo.tokens)
                    .create_worklog(&me, &jira_issue.id, description, til_now, start.to_owned())
                    .await?;

                if result {
                    timers.save(&config_root)?;
                }

                show_worklog_result(
                    result,
                    &{
                        let mut time_str = String::new();

                        let hours = til_now / 3600;
                        if hours > 0 {
                            time_str += &format!("{}h", hours);
                        }
                        let minutes = til_now % 3600 / 60;
                        if minutes > 0 {
                            time_str += &format!("{}m", minutes);
                        }
                        let seconds = til_now % 60;
                        if seconds > 0 {
                            time_str += &format!("{}s", seconds);
                        }

                        time_str
                    },
                    &jira_issue,
                );
            } else {
                timers.save(&config_root)?;
                println!("Cancelled timer");
            }
        }
    }

    Ok(())
}
