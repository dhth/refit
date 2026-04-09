use super::process;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error(transparent)]
    Process(#[from] process::ProcessError),
    #[error(
        "git operation `{operation}` failed with status {status}:\nstdout:\n{stdout}\nstderr:\n{stderr}",
        status = format_status(.status)
    )]
    Failed {
        operation: &'static str,
        status: Option<i32>,
        stdout: String,
        stderr: String,
    },
}

pub fn clone_repo(repo: &str, target_dir: &Path) -> Result<(), GitError> {
    let target_dir = target_dir.to_string_lossy();
    let output = process::run(
        "git",
        &["clone", "--depth", "1", repo, target_dir.as_ref()],
        None,
    )?;

    let status = output.status_code();
    if !output.success() {
        let (stdout, stderr) = output.into_parts();

        return Err(GitError::Failed {
            operation: "clone",
            status,
            stdout,
            stderr,
        });
    }

    Ok(())
}

pub fn diff_paths(left_path: &Path, right_path: &Path) -> Result<(), GitError> {
    let left_path = left_path.to_string_lossy();
    let right_path = right_path.to_string_lossy();
    let status = process::run_streaming(
        "git",
        &[
            "diff",
            "--no-index",
            left_path.as_ref(),
            right_path.as_ref(),
        ],
        None,
    )?;

    match status.code() {
        Some(0 | 1) => Ok(()),
        status => Err(GitError::Failed {
            operation: "diff --no-index",
            status,
            stdout: String::new(),
            stderr: String::new(),
        }),
    }
}

fn format_status(status: &Option<i32>) -> String {
    match status {
        Some(status) => status.to_string(),
        None => String::from("unknown"),
    }
}
