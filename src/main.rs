mod cli;
mod command;
mod config;
mod runner;

use crate::config::AppSettings;
use crate::runner::process;
use cli::Cli;

use clap::Parser;
use std::fs;
fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut log = args.log;

    if args.verbose {
        log = "trace".to_string();
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&log)).init();

    let settings: AppSettings = serde_json::from_str(&fs::read_to_string(&args.config)?)?;

    process(args.path, &settings, args.check, args.dry_run)
}
