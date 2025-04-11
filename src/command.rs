use log::debug;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn prepare_command(command_str: &str) -> Command {
    let (shell, shell_arg) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };
    let mut cmd = Command::new(shell);
    cmd.arg(shell_arg)
        .arg(command_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

pub fn run_command_with_input(
    command: &str,
    input: &str,
) -> anyhow::Result<(String, String, bool)> {
    let mut cmd = prepare_command(command);
    cmd.stdin(Stdio::piped());

    debug!("Executing command {:?}", command);

    let mut child = cmd.spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    ))
}

pub fn run_command_with_file(command: &str) -> anyhow::Result<(String, String, bool)> {
    let mut cmd = prepare_command(command);

    debug!("Executing command {:?}", command);

    let output = cmd.output()?;

    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    ))
}

pub fn expand_command_template(template: &str, file: &Path, lang: &str) -> String {
    template
        .replace("{file}", file.to_str().unwrap_or(""))
        .replace("{lang}", lang)
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
}
