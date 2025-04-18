mod cli;
mod command;
mod config;
mod runner;

use crate::config::AppSettings;
use crate::runner::process;
use anyhow::Result;
use cli::Cli;

use clap::Parser;
use std::fs;

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut log = args.log;

    if args.verbose {
        log = "trace".to_string();
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&log)).init();

    let settings: AppSettings = toml::from_str(&fs::read_to_string(&args.config)?)?;

    let results = process(args.path, &settings, args.check);

    if results.is_err() {
        std::process::exit(1);
    }

    Ok(())
}
