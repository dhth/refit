use crate::domain::{Config, RunPlan, RunSource, RunUpdate};
use crate::ops::git::{self, GitError};
use crate::ops::process::{self, ProcessError, ProcessOutput};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("no update found matching regex `{0}`")]
    NoUpdates(regex::Regex),
    #[error("couldn't ask for confirmation: {0}")]
    CouldntAskForConfirmation(std::io::Error),
    #[error("couldn't create temporary directory for cloning remote repository: {0}")]
    CouldntCreateTempDir(#[from] std::io::Error),
    #[error("there were errors while running for {0} sources")]
    SourceErrors(usize),
}

pub fn handle_run(
    config: Config,
    regex: regex::Regex,
    skip_confirmation: bool,
) -> Result<(), RunError> {
    let plan = RunPlan::create(config, &regex).ok_or(RunError::NoUpdates(regex))?;
    if !skip_confirmation {
        let proceed = ask_for_confirmation(&plan).map_err(RunError::CouldntAskForConfirmation)?;
        if !proceed {
            println!("aborted");
            return Ok(());
        }
    }

    let clone_dir = tempfile::tempdir()?;

    let mut num_source_errors = 0;
    println!();

    for source in plan.sources() {
        println!("running for source: {}", source.repo());

        if let Err(error) = handle_run_source(source, clone_dir.path()) {
            num_source_errors += 1;
            eprintln!("  error: {error}");
        }
    }

    if num_source_errors == 0 {
        Ok(())
    } else {
        Err(RunError::SourceErrors(num_source_errors))
    }
}

#[derive(Debug, thiserror::Error)]
enum RunSourceError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("{0} updates failed")]
    UpdateErrors(usize),
}

fn handle_run_source(source: &RunSource, clone_dir: &Path) -> Result<(), RunSourceError> {
    let source_dir = clone_dir.join(source.name());
    git::clone_repo(source.repo(), &source_dir)?;

    let mut num_errors = 0;
    for update in source.updates() {
        match handle_run_update(update, &source_dir) {
            Ok(_) => println!(
                "    ✅ {}: ({} -> {})",
                update.name(),
                update.source_path().to_string_lossy(),
                update.target_path().to_string_lossy(),
            ),
            Err(error) => {
                num_errors += 1;
                eprintln!("    ❌ {}: {error}", update.name());
            }
        }
    }

    if num_errors == 0 {
        Ok(())
    } else {
        Err(RunSourceError::UpdateErrors(num_errors))
    }
}

#[derive(Debug, thiserror::Error)]
enum RunUpdateError {
    #[error("source path `{0}` does not exist in remote repository")]
    SourcePathMissing(PathBuf),
    #[error("couldn't determine metadata for the path `{0}`: {1}")]
    CouldntDetermineMetadata(PathBuf, std::io::Error),
    #[error("target path `{0}` is a symlink")]
    TargetPathIsASymlink(PathBuf),
    #[error("couldn't delete previous contents: {0}")]
    CouldntDeletePreviousContents(std::io::Error),
    #[error("couldn't create parent directory `{0}`: {1}")]
    CouldntCreateParentDir(PathBuf, std::io::Error),
    #[error("couldn't run copy command: {0}")]
    CouldntRunCopyCommand(#[from] ProcessError),
    #[error("copy command failed: {0}")]
    CopyCommandFailed(ProcessOutput),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

fn handle_run_update(update: &RunUpdate, clone_root: &Path) -> Result<(), RunUpdateError> {
    let source_path = clone_root.join(update.source_path());
    let source_metadata = match source_path.metadata() {
        Ok(m) => m,
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => {
                return Err(RunUpdateError::SourcePathMissing(
                    update.source_path().to_path_buf(),
                ));
            }
            _ => {
                return Err(RunUpdateError::CouldntDetermineMetadata(
                    source_path.to_path_buf(),
                    error,
                ));
            }
        },
    };

    let target_path = update.target_path();

    let target_metadata = match target_path.symlink_metadata() {
        Ok(m) => Some(m),
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => None,
            _ => {
                return Err(RunUpdateError::CouldntDetermineMetadata(
                    target_path.to_path_buf(),
                    error,
                ));
            }
        },
    };

    if let Some(metadata) = target_metadata {
        if metadata.is_symlink() {
            return Err(RunUpdateError::TargetPathIsASymlink(
                target_path.to_path_buf(),
            ));
        }

        if metadata.is_dir() {
            std::fs::remove_dir_all(target_path)
        } else {
            std::fs::remove_file(target_path)
        }
        .map_err(RunUpdateError::CouldntDeletePreviousContents)?;
    }

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| RunUpdateError::CouldntCreateParentDir(parent.to_path_buf(), e))?;
    }

    let cp_args = {
        let mut cp_args = vec![];
        if source_metadata.is_dir() {
            cp_args.push("-R".to_string());
        }
        cp_args.push("--".to_string());
        cp_args.push(source_path.to_string_lossy().to_string());
        cp_args.push(target_path.to_string_lossy().to_string());

        cp_args
    };

    let copy_output = process::run("cp", cp_args.as_ref(), None)?;
    if !copy_output.success() {
        return Err(RunUpdateError::CopyCommandFailed(copy_output));
    }

    Ok(())
}

fn ask_for_confirmation(plan: &RunPlan) -> Result<bool, std::io::Error> {
    print!(
        r#"refit will make the following updates:

---
{}---

Continue? Type "yes" to proceed.
"#,
        plan
    );

    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() == "yes" {
        Ok(true)
    } else {
        Ok(false)
    }
}
