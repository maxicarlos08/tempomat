use clap::Parser;
use std::{env, fs, path::PathBuf};
use tempomat::{
    args::{CLISubcommand, TempomatCLI},
    config::{APITokens, Config, Saveable},
    dirs,
    error::TempomatError,
};

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

    let mut config = Config::try_read(&config_root).ok();
    let mut tokens = APITokens::try_read(&config_root).ok();

    match args.command {
        CLISubcommand::Log {
            time,
            message,
            issue_id,
        } => {
            let (Some(config), Some(mut tokens)) = (config.take(), tokens.take()) else {
                Err(TempomatError::MissingConfigurations)?
            };
            // Ensure tokens arent outdated
            tokens.refresh_tokens().await?;

            todo!("Log time")
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
