use crate::args::{Args, RefitCommand};
use crate::errors::AppError;
use clap::Parser;

pub fn run() -> Result<(), AppError> {
    let args = Args::parse();
    let config = crate::config::load()?;

    match args.command {
        RefitCommand::Run {
            regex,
            skip_confirmation,
        } => {
            crate::cmds::handle_run(config, regex, skip_confirmation)?;
        }
        RefitCommand::Diff { id } => {
            crate::cmds::handle_diff(config, id)?;
        }
    }

    Ok(())
}
