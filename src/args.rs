use clap::{Parser, Subcommand};
use regex::Regex;

/// refit lets you replace local paths with contents from remote git repositories
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: RefitCommand,
}

#[derive(Subcommand, Debug)]
pub enum RefitCommand {
    /// Run all updates whose names match the regex
    Run {
        /// Regular expression to match updates (in the form "source-name/update-name")
        #[arg(value_name = "REGEX", value_parser = parse_regex)]
        regex: Regex,
        /// Skip confirmation and run matched updates immediately
        #[arg(short = 'y', long = "yes")]
        skip_confirmation: bool,
    },
    /// Show the diff for exactly one update
    Diff {
        /// ID of the update to run for, in the form "source-name/update-name"
        #[arg(value_name = "ID")]
        id: String,
    },
}

fn parse_regex(value: &str) -> Result<Regex, String> {
    Regex::new(value).map_err(|e| format!("invalid regex: {}", e))
}
