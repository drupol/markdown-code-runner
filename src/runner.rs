use crate::command::{expand_command_template, run_command_with_file, run_command_with_input};
use crate::config::{AppSettings, InputMode, OutputMode};

use anyhow::anyhow;
use log::{debug, error, info, warn};
use pulldown_cmark::{CodeBlockKind, Event, Parser as MdParser, Tag};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use walkdir::WalkDir;

pub fn process(
    path: PathBuf,
    config: &AppSettings,
    check_only: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    collect_markdown_files(&path)
        .par_iter()
        .map(|file| process_markdown_file(file, config, check_only, dry_run))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

fn collect_markdown_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .map(|e| e.into_path())
            .collect()
    }
}

fn process_markdown_file(
    path: &Path,
    config: &AppSettings,
    check_only: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;
    let original_lines: Vec<&str> = content.lines().collect();

    let mut parser = MdParser::new(&content).into_offset_iter();
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut global_result_mismatch = false;
    let mut global_result_success = true;

    while let Some((event, range)) = parser.next() {
        let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(block_code_headers))) = event else {
            continue;
        };

        let mut parts = block_code_headers.split_whitespace();
        let lang = parts.next().unwrap_or_default();
        let attributes: Vec<&str> = parts.collect();

        if attributes.contains(&"mdcr-skip") {
            debug!("mdcr-skip has been found in the code block header, skipping...");
            continue;
        }

        let mut code = String::new();
        let start_offset = range.start;
        let mut end_offset = range.end;

        for (ev, r) in parser.by_ref() {
            match ev {
                Event::Text(text) => {
                    code.push_str(&text);
                    end_offset = r.end;
                }
                Event::End(_) => {
                    end_offset = r.end;
                    break;
                }
                _ => {}
            }
        }

        let mut start_line = content[..start_offset].lines().count();
        let end_line = content[..end_offset].lines().count();

        for cfg in config.presets.values() {
            if cfg.language.trim() != lang.trim() {
                continue;
            }

            debug!(
                "Processing preset for language `{}` in mode `{:?}`",
                lang, cfg.mode
            );

            let (output, stderr, success) = match cfg.input_mode {
                InputMode::String => run_command_with_input(&cfg.command, &code)?,
                InputMode::File => {
                    let tmp = NamedTempFile::new()?;
                    fs::write(tmp.path(), &code)?;
                    let cmd = expand_command_template(&cfg.command, tmp.path(), &cfg.language);
                    run_command_with_file(&cmd)?
                }
            };

            global_result_success = global_result_success && success;

            if matches!(cfg.mode, OutputMode::Check) {
                if success {
                    continue;
                }

                let msg = format!(
                    "Command for language `{}` failed (exit != 0) in file `{}`:\n{}",
                    cfg.language,
                    path.display(),
                    stderr.trim()
                );

                if dry_run {
                    warn!("{msg}");
                    continue;
                }

                if check_only {
                    continue;
                }
                return Err(anyhow!("{msg}"));
            }

            if matches!(cfg.mode, OutputMode::Replace) {
                if !success {
                    let msg = format!(
                        "Command for language `{}` failed (exit != 0) in file `{}`:\n{}",
                        cfg.language,
                        path.display(),
                        stderr.trim()
                    );

                    if dry_run {
                        warn!("{msg}");
                        continue;
                    }

                    if check_only {
                        error!("{msg}");
                        continue;
                    }

                    break;
                }

                let mismatch = output.trim() != code.trim();
                global_result_mismatch = global_result_mismatch || mismatch;

                if !mismatch {
                    debug!("Skipping code block, it seems it is already processed...");
                    continue;
                };

                if dry_run {
                    warn!("Code block mismatch detected in: {}", path.display());
                    continue;
                }

                if check_only {
                    error!("Code block mismatch detected in: {}", path.display());
                    break;
                }

                let indent = original_lines
                    .get(start_line)
                    .map(|line| {
                        line.chars()
                            .take_while(|c| c.is_whitespace())
                            .collect::<String>()
                    })
                    .unwrap_or_default();

                let replacement = std::iter::once(format!("```{}", block_code_headers))
                    .chain(output.lines().map(|l| l.to_string()))
                    .chain(std::iter::once("```".to_string()))
                    .map(|l| format!("{indent}{l}"))
                    .collect();

                if !indent.is_empty() && start_line > 0 {
                    start_line -= 1;
                }

                replacements.push((start_line, end_line, replacement));
            }
        }
    }

    if check_only {
        if !global_result_success {
            return Err(anyhow!("Error(s) while executing commands",));
        }

        if global_result_mismatch {
            return Err(anyhow!(
                "Code block mismatch detected in: {}",
                path.display()
            ));
        }

        return Ok(());
    }

    if dry_run {
        if !global_result_success {
            info!("Error(s) while checking some code blocks in files");
        }

        if global_result_mismatch {
            info!("File would be updated: {}", path.display());
        }

        return Ok(());
    }

    let mut updated_lines = original_lines
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    for (start, end, new_lines) in replacements.into_iter().rev() {
        updated_lines.splice(start..end, new_lines);
    }

    info!("Code block mismatch detected in: {}", path.display());

    fs::write(path, updated_lines.join("\n") + "\n")?;
    info!("File updated: {}", path.display());
    Ok(())
}
