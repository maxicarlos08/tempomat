use crate::config::Saveable;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
pub struct TempoTimers(pub HashMap<String, NaiveDateTime>);

impl Saveable for TempoTimers {
    fn path(root: &std::path::Path) -> std::path::PathBuf {
        root.join("timers.ron")
    }
}
