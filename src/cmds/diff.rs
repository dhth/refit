use crate::domain::{Config, DiffPlan};
use crate::ops::git::{self, GitError};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    #[error("no update found with id `{0}`")]
    UnknownUpdate(String),
    #[error("couldn't create temporary directory for cloning remote repository: {0}")]
    CouldntCreateTempDir(#[from] std::io::Error),
    #[error("source path `{path}` does not exist in remote repository `{repo}`")]
    SourcePathMissing { repo: String, path: String },
    #[error(
        "source path doesn't exist in repo; couldn't create new empty directory for diffing: {0}"
    )]
    CouldntCreateNewEmptyDir(std::io::Error),
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub fn handle_diff(config: Config, id: String) -> Result<(), DiffError> {
    let plan = DiffPlan::create(&config, &id).ok_or(DiffError::UnknownUpdate(id))?;
    let clone_dir = tempfile::tempdir()?;

    git::clone_repo(&plan.repo, clone_dir.path())?;
    run_diff_with_cloned_repo(&plan, clone_dir.path())
}

fn run_diff_with_cloned_repo(plan: &DiffPlan, clone_dir: &Path) -> Result<(), DiffError> {
    let source_path = clone_dir.join(&plan.source_path);

    if !source_path.exists() {
        return Err(DiffError::SourcePathMissing {
            repo: plan.repo.clone(),
            path: plan.source_path.display().to_string(),
        });
    }

    if plan.target_path.exists() {
        git::diff_paths(&plan.target_path, &source_path)?;
    } else {
        let empty_temp_dir = tempfile::tempdir().map_err(DiffError::CouldntCreateNewEmptyDir)?;

        git::diff_paths(empty_temp_dir.path(), &source_path)?;
    }

    Ok(())
}
