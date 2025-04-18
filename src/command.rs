use log::debug;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::NamedTempFile;

use crate::config::{InputMode, PresetConfig};

pub fn run_command(cfg: &PresetConfig, input: &str) -> anyhow::Result<(Command, Output)> {
    match cfg.input_mode {
        InputMode::Stdin => run_command_with_stdin(&cfg.command, input, &cfg.language),
        InputMode::File => run_command_with_file(&cfg.command, input, &cfg.language),
    }
}

fn run_command_with_stdin(
    command_template: &[String],
    input: &str,
    lang: &str,
) -> anyhow::Result<(Command, Output)> {
    let args = expand_command_vec(command_template, None, lang);
    let mut cmd = Command::new(&args[0]);

    cmd.args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    Ok((cmd, output))
}

fn run_command_with_file(
    command_template: &[String],
    input: &str,
    lang: &str,
) -> anyhow::Result<(Command, Output)> {
    let tmp = NamedTempFile::new()?;
    fs::write(tmp.path(), input)?;
    let args = expand_command_vec(command_template, Some(tmp.path()), lang);

    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    debug!("Executing command {:?}", args);
    let output = cmd.output()?;

    Ok((cmd, output))
}

fn expand_command_vec(template: &[String], file: Option<&Path>, lang: &str) -> Vec<String> {
    template
        .iter()
        .map(|arg| {
            let replaced = arg.replace("{lang}", lang);
            if let Some(file) = file {
                replaced
                    .replace("{file}", file.to_str().unwrap_or("{file}"))
                    .replace(
                        "{basename}",
                        file.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                    )
                    .replace(
                        "{dirname}",
                        file.parent().and_then(|s| s.to_str()).unwrap_or(""),
                    )
                    .replace(
                        "{suffix}",
                        file.extension().and_then(|s| s.to_str()).unwrap_or(""),
                    )
                    .replace("{tmpdir}", std::env::temp_dir().to_str().unwrap_or(""))
            } else {
                replaced
            }
        })
        .collect()
}

pub fn command_to_string(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join(" ");
    format!("{} {}", program, args)
}
