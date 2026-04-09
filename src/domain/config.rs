use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

const NAME_REGEX: &str = "^[a-z0-9_-]+$";

#[derive(Deserialize)]
pub struct RawUpdate {
    pub name: String,
    pub path: String,
    pub target: String,
}

#[derive(Deserialize)]
pub struct RawSource {
    pub name: String,
    pub repo: String,
    pub updates: Vec<RawUpdate>,
}

#[derive(Deserialize)]
pub struct RawConfig {
    pub sources: Vec<RawSource>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Config {
    pub sources: Vec<Source>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Source {
    pub name: String,
    pub repo: String,
    pub updates: Vec<Update>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Update {
    pub name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateConfigError {
    #[error("{0}")]
    InvalidField(String),
    #[error("duplicate source name `{0}`")]
    DuplicateSourceName(String),
    #[error(
        "multiple sources defined for the repo `{0}`; each repo must appear in exactly one source group"
    )]
    DuplicateRepo(String),
    #[error("duplicate update name `{0}` in source `{1}`")]
    DuplicateUpdateName(String, String),
}

impl TryFrom<RawConfig> for Config {
    type Error = ValidateConfigError;

    fn try_from(value: RawConfig) -> Result<Self, Self::Error> {
        let mut seen_source_names: HashSet<String> = HashSet::new();
        let mut seen_repos: HashSet<String> = HashSet::new();
        let mut sources = Vec::new();

        #[allow(clippy::expect_used)]
        let name_regex = Regex::new(NAME_REGEX).expect("name regex should've been created");

        for (source_index, raw_source) in value.sources.into_iter().enumerate() {
            let source_name = raw_source.name.trim().to_string();
            if source_name.is_empty() {
                return Err(ValidateConfigError::InvalidField(format!(
                    "source at index {} has an empty name (index starts at 1)",
                    source_index + 1
                )));
            }

            if !name_regex.is_match(&source_name) {
                return Err(ValidateConfigError::InvalidField(format!(
                    "source at index {} has an invalid name `{source_name}` (should conform to the pattern `{NAME_REGEX}`)",
                    source_index + 1
                )));
            }

            if !seen_source_names.insert(source_name.clone()) {
                return Err(ValidateConfigError::DuplicateSourceName(source_name));
            }

            let repo = raw_source.repo.trim().to_string();
            if repo.is_empty() {
                return Err(ValidateConfigError::InvalidField(format!(
                    "source `{source_name}` has an empty repository"
                )));
            }

            if !seen_repos.insert(repo.clone()) {
                return Err(ValidateConfigError::DuplicateRepo(repo));
            }

            let mut seen_update_names: HashSet<String> = HashSet::new();
            let mut updates = Vec::new();

            for (update_index, raw_update) in raw_source.updates.into_iter().enumerate() {
                let update_name = raw_update.name.trim().to_string();
                if update_name.is_empty() {
                    return Err(ValidateConfigError::InvalidField(format!(
                        "name for update at index {} in source `{source_name}` is empty (index starts at 1)",
                        update_index + 1
                    )));
                }

                if !name_regex.is_match(&update_name) {
                    return Err(ValidateConfigError::InvalidField(format!(
                        "update at index {} in source `{source_name}` has an invalid name `{update_name}` (should conform to the pattern `{NAME_REGEX}`)",
                        update_index + 1
                    )));
                }

                if !seen_update_names.insert(update_name.clone()) {
                    return Err(ValidateConfigError::DuplicateUpdateName(
                        update_name,
                        source_name.clone(),
                    ));
                }

                let path = raw_update.path.trim().to_string();
                if path.is_empty() {
                    return Err(ValidateConfigError::InvalidField(format!(
                        "path for update `{source_name}/{update_name}` is empty"
                    )));
                }

                let source_path = validate_relative_path(&path, "path", &source_name, &update_name)
                    .map_err(ValidateConfigError::InvalidField)?;

                let target_str = raw_update.target.trim().to_string();
                if target_str.is_empty() {
                    return Err(ValidateConfigError::InvalidField(format!(
                        "target for update `{source_name}/{update_name}` is empty"
                    )));
                }

                let target_path =
                    validate_relative_path(&target_str, "target", &source_name, &update_name)
                        .map_err(ValidateConfigError::InvalidField)?;

                updates.push(Update {
                    name: update_name,
                    source_path,
                    target_path,
                });
            }

            sources.push(Source {
                name: source_name,
                repo,
                updates,
            });
        }

        Ok(Self { sources })
    }
}

fn validate_relative_path(
    value: &str,
    field_name: &str,
    source_name: &str,
    update_name: &str,
) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);

    if path.is_absolute() {
        return Err(format!(
            "{field_name} for update `{source_name}/{update_name}` (`{value}`) is not a relative path"
        ));
    }

    if Path::new(value)
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(format!(
            "{field_name} for update `{source_name}/{update_name}` (`{value}`) must not contain `..`"
        ));
    }

    Ok(path)
}
