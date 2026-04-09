use super::{Config, Source};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct DiffPlan {
    pub repo: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
}

impl DiffPlan {
    pub fn create(config: &Config, update_id: &str) -> Option<Self> {
        for source in &config.sources {
            for update in &source.updates {
                if update_id == format!("{}/{}", source.name, update.name) {
                    return Some(Self {
                        repo: source.repo.clone(),
                        source_path: update.source_path.clone(),
                        target_path: update.target_path.clone(),
                    });
                }
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct RunUpdate {
    name: String,
    source_path: PathBuf,
    target_path: PathBuf,
}

impl RunUpdate {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn target_path(&self) -> &Path {
        &self.target_path
    }
}

#[derive(Debug)]
pub struct RunSource {
    name: String,
    repo: String,
    updates: Vec<RunUpdate>,
}

impl RunSource {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn updates(&self) -> &[RunUpdate] {
        &self.updates
    }
}

#[derive(Debug)]
pub struct RunPlan {
    sources: Vec<RunSource>,
}

impl RunPlan {
    pub fn create(config: Config, regex: &regex::Regex) -> Option<Self> {
        let sources = config
            .sources
            .into_iter()
            .filter_map(|source| Self::select_source(source, regex))
            .collect::<Vec<_>>();

        if sources.is_empty() {
            None
        } else {
            Some(Self { sources })
        }
    }

    fn select_source(source: Source, regex: &regex::Regex) -> Option<RunSource> {
        let Source {
            name: source_name,
            repo,
            updates,
        } = source;

        let matches_updates = updates
            .into_iter()
            .filter_map(|update| {
                let update_id = format!("{}/{}", &source_name, &update.name);
                if regex.is_match(&update_id) {
                    Some(RunUpdate {
                        name: update.name,
                        source_path: update.source_path,
                        target_path: update.target_path,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if matches_updates.is_empty() {
            None
        } else {
            Some(RunSource {
                name: source_name,
                repo,
                updates: matches_updates,
            })
        }
    }

    pub fn sources(&self) -> &[RunSource] {
        &self.sources
    }
}

impl std::fmt::Display for RunPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for source in &self.sources {
            writeln!(f, "{}", source.repo)?;
            for update in &source.updates {
                writeln!(
                    f,
                    "    {} -> {}",
                    update.source_path.to_string_lossy(),
                    update.target_path.to_string_lossy()
                )?;
            }
        }

        Ok(())
    }
}
