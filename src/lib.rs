use directories::ProjectDirs;

pub mod args;
pub mod config;
pub mod error;
pub mod git;
pub mod jira;
pub mod tempo;

pub fn dirs() -> Result<ProjectDirs, error::TempomatError> {
    ProjectDirs::from("de", "maxicarlos", "tempomat").ok_or(error::TempomatError::NoProjectDirs)
}
