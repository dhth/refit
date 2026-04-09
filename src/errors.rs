use crate::cmds::{DiffError, RunError};
use crate::config::ConfigError;

const SAMPLE_CONFIG: &str = include_str!("assets/sample-config.yml");

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Run(#[from] RunError),
    #[error(transparent)]
    Diff(#[from] DiffError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    pub fn follow_up(&self) -> Option<String> {
        match self {
            AppError::Config(config_error) => match config_error {
                ConfigError::NotFound => Some(format!(
                    r#"
Tip: create a config file ".refit.yml" that looks like the following:

---
{SAMPLE_CONFIG}---
"#
                )),
                ConfigError::Parse(_) => Some(format!(
                    r#"
Tip: a valid config file looks like the following:

---
{SAMPLE_CONFIG}---
"#
                )),
                _ => None,
            },
            _ => None,
        }
    }
}
