use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    arg_required_else_help = true,
    name = "Markdown Code Runner",
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    about = clap::crate_description!()
)]
pub struct Cli {
    /// Path to the Markdown file or directory
    pub path: PathBuf,
    /// Path to the config JSON file
    #[arg(long)]
    pub config: PathBuf,
    /// Run in check mode (do not write changes)
    #[arg(long)]
    pub check: bool,
    /// Simulate changes, but don't write files
    #[arg(long = "dry-run")]
    pub dry_run: bool,
    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "warn")]
    pub log: String,
    /// Verbose mode (set the log level to `trace`)
    #[arg(long)]
    pub verbose: bool,
}
