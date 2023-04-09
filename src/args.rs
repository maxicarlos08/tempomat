use crate::jira::types::JiraIssueKey;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct TempomatCLI {
    /// Override configuration root path value, can also be override using $TEMPOMAT_ROOT
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: CLISubcommand,
}

#[derive(Subcommand, Debug)]
pub enum CLISubcommand {
    /// Create a new time log
    Log {
        /// Amount of time to log (XhYmZs)
        #[arg(value_parser = parsers::parse_arg)]
        time: (usize, String),
        /// Description of the time log
        #[arg(short, long)]
        description: Option<String>,
        /// Jira issue ID to log to
        #[arg(short, long, value_parser = parsers::parse_issue_id)]
        issue: Option<JiraIssueKey>,
    },
    /// Log in to Tempo and Jira
    Login {
        /// Name of the atlassian instance you have tempo installed to
        #[arg(long)]
        atlassian_instance: String,
    },
    /// Start a new timer
    Start {
        /// Issue to start a timer for
        #[arg(short, long, value_parser = parsers::parse_issue_id)]
        issue: Option<JiraIssueKey>,
    },
    /// End a timer
    Stop {
        /// Clear the timer without submitting it
        #[arg(short, long)]
        no_submit: bool,
        /// Description of the time log
        #[arg(short, long)]
        description: Option<String>,
        /// Issue of the timer
        #[arg(short, long, value_parser = parsers::parse_issue_id)]
        issue: Option<JiraIssueKey>,
    },
}

mod parsers {
    use crate::jira::{parse_issue_key, types::JiraIssueKey};
    use nom::{
        bytes::complete::{tag, take_while},
        combinator::map_res,
        IResult,
    };

    pub fn parse_issue_id(id: &str) -> Result<JiraIssueKey, String> {
        match parse_issue_key(id) {
            Ok((_, key)) => Ok(key),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn parse_arg(time: &str) -> Result<(usize, String), String> {
        let original = time.to_owned();
        match parse_duration(time) {
            Ok(("", duration)) => Ok((duration, original)),
            Ok((remaining, _)) => Err(format!(
                "Could not parse this remaining duration fragment: {remaining}"
            )),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn parse_duration(duration: &str) -> IResult<&str, usize> {
        let (duration, hours) = parse_time(duration, "h", 60 * 60).unwrap_or((duration, 0));
        let (duration, minutes) = parse_time(duration, "m", 60).unwrap_or((duration, 0));
        let (duration, seconds) = parse_time(duration, "s", 1).unwrap_or((duration, 0));

        Ok((duration, hours + minutes + seconds))
    }

    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    // TODO: Get rid of repetition

    fn parse_time<'a>(
        i: &'a str,
        end_tag: &'static str,
        multiplier: usize,
    ) -> IResult<&'a str, usize> {
        let mut duration = map_res(take_while(is_digit), |i: &str| i.parse::<usize>())(i)?;
        (duration.0, _) = tag(end_tag)(duration.0)?;

        Ok((duration.0, duration.1 * multiplier))
    }

    #[cfg(test)]
    mod test {
        use super::parse_arg;

        #[test]
        fn test_correct_times() {
            assert_eq!(parse_arg("1m").unwrap().0, 60);
            assert_eq!(parse_arg("6h7s").unwrap().0, 21607);
            assert_eq!(parse_arg("1h30m").unwrap().0, 5400);
        }

        #[test]
        fn test_incorrect_times() {
            assert!(parse_arg("1s2h").is_err());
            assert!(parse_arg("6d3s").is_err());
        }
    }
}
