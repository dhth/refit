use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

#[derive(Debug)]
pub struct ProcessOutput {
    command: String,
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl ProcessOutput {
    pub fn success(&self) -> bool {
        self.status.success()
    }

    pub fn status_code(&self) -> Option<i32> {
        self.status.code()
    }

    pub fn into_parts(self) -> (String, String) {
        (self.stdout, self.stderr)
    }
}

impl std::fmt::Display for ProcessOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"command: {}
success: {}
exit code: {}
stdout:
{}
stderr:
{}"#,
            self.command,
            self.status.success(),
            self.status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_default(),
            self.stdout,
            self.stderr,
        )
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to run `{command}`: {source}")]
pub struct ProcessError {
    command: String,
    #[source]
    source: std::io::Error,
}

pub fn run<S>(
    command: &str,
    args: &[S],
    working_dir: Option<&Path>,
) -> Result<ProcessOutput, ProcessError>
where
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(command);
    cmd.args(args);

    if let Some(working_dir) = working_dir {
        cmd.current_dir(working_dir);
    }

    let output = cmd.output().map_err(|source| ProcessError {
        command: render_command(command, args),
        source,
    })?;

    Ok(ProcessOutput {
        command: render_command(command, args),
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

pub fn run_streaming(
    command: &str,
    args: &[&str],
    working_dir: Option<&Path>,
) -> Result<ExitStatus, ProcessError> {
    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(working_dir) = working_dir {
        cmd.current_dir(working_dir);
    }

    cmd.status().map_err(|source| ProcessError {
        command: render_command(command, args),
        source,
    })
}

fn render_command<S>(command: &str, args: &[S]) -> String
where
    S: AsRef<OsStr>,
{
    let mut rendered = String::from(command);

    for arg in args {
        rendered.push(' ');
        if let Some(str) = arg.as_ref().to_str() {
            rendered.push_str(str);
        }
    }

    rendered
}
