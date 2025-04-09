use log::debug;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_command_with_input(
    command: &str,
    input: &str,
) -> anyhow::Result<(String, String, bool)> {
    let (shell, shell_arg) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let mut binding = Command::new(shell);
    let command = binding
        .arg(shell_arg)
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    debug!("Executing command {:?}", command);

    let mut child = command.spawn()?;

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
    let (shell, shell_arg) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let mut binding = Command::new(shell);
    let command = binding
        .arg(shell_arg)
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    debug!("Executing command {:?}", command);

    let output = command.output()?;

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
