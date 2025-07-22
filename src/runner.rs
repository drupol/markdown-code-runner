use crate::config::{AppSettings, OutputMode, PresetConfig};

use crate::codeblock::{CodeBlock, CodeBlockIterator, CodeBlockProcessingResult};
use crate::command::{command_to_string, run_command};

use anyhow::anyhow;
use anyhow::{Context, Result};
use log::{debug, error, info};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use walkdir::WalkDir;

pub fn process(path: PathBuf, config: &AppSettings, check_only: bool) -> anyhow::Result<()> {
    let files = collect_markdown_files(&path)?;

    let results: Vec<anyhow::Result<()>> = files
        .iter()
        .map(|file| process_markdown_file(file, config, check_only))
        .collect();

    if results.iter().any(Result::is_err) {
        return Err(anyhow!(""));
    }

    Ok(())
}

fn process_markdown_file(
    path: &Path,
    config: &AppSettings,
    check_only: bool,
) -> anyhow::Result<()> {
    let blocks: Vec<_> = CodeBlockIterator::new(path)?.collect();

    let results: Vec<CodeBlockProcessingResult> = blocks
        .iter()
        .rev()
        .map(|block| process_block(path, config, block, check_only))
        .collect();

    let file_has_command_failures = results.iter().any(|r| r.had_command_failure);
    let file_has_mismatches = results.iter().any(|r| r.had_mismatch);
    let all_replacements: Vec<_> = results.into_iter().flat_map(|r| r.replacements).collect();

    if file_has_command_failures {
        return Err(anyhow!(
            "One or more commands failed in file `{}`",
            path.display()
        ));
    }

    if check_only && file_has_mismatches {
        return Err(anyhow!(
            "Checking some files failed, see the logs for details.",
        ));
    }

    if all_replacements.is_empty() {
        debug!("No changes needed for file `{}`", path.display());
        return Ok(());
    }

    apply_replacements(all_replacements)
}

fn process_block(
    path: &Path,
    config: &AppSettings,
    block: &CodeBlock,
    check_only: bool,
) -> CodeBlockProcessingResult {
    let mut replacements = Vec::new();
    let mut had_command_failure = false;
    let mut had_mismatch = false;

    for (preset, preset_cfg) in &config.presets {
        if preset_cfg.language.trim() != block.lang {
            debug!(
                "Skipping preset `{}` for language `{}` in `{}`",
                preset,
                block.lang,
                path.display()
            );
            continue;
        }

        debug!(
            "Processing file `{}` and preset `{}` for language `{}` in `{:?}` mode...",
            block.path.display(),
            preset,
            block.lang,
            preset_cfg.output_mode
        );

        match run_command(preset_cfg, &block.code) {
            Ok((command, output)) => {
                if !output.status.success() {
                    error!(
                        "The command `{}` returned a non-zero exit status ({}) for preset `{}` in `{}:{}-{}`, `{}`",
                        command_to_string(&command),
                        output.status.code().unwrap_or(-1),
                        preset,
                        path.display(),
                        block.start_line,
                        block.end_line,
                        String::from_utf8_lossy(&output.stderr).trim()
                    );
                    had_command_failure = true;
                    continue;
                }

                match handle_preset_result(&output, preset, preset_cfg, block, check_only) {
                    Ok(Some(replacement)) => {
                        had_mismatch = true;
                        replacements.push(replacement);
                    }
                    Ok(None) => {}
                    Err(_) => {
                        had_mismatch = true;
                    }
                }
            }
            Err(e) => {
                error!(
                    "Error executing command for preset `{}` in `{}`: {}",
                    preset,
                    path.display(),
                    e
                );
                had_command_failure = true;
            }
        }
    }

    CodeBlockProcessingResult {
        replacements,
        had_command_failure,
        had_mismatch,
    }
}

fn apply_replacements(replacements: Vec<CodeBlock>) -> Result<()> {
    let mut replacements_by_file: HashMap<PathBuf, Vec<CodeBlock>> = HashMap::new();

    for block in replacements {
        replacements_by_file
            .entry(block.path.clone())
            .or_default()
            .push(block);
    }

    replacements_by_file
        .into_par_iter()
        .map(|(file_path, codeblocks)| -> Result<_> {
            let mut file_lines: Vec<String> = fs::read_to_string(&file_path)?
                .lines()
                .map(String::from)
                .collect();

            for codeblock in codeblocks {
                let bounded_end = codeblock.end_line.min(file_lines.len());
                let bounded_start = codeblock.start_line.min(bounded_end);
                debug!(
                    "Applying replacement lines `{}:{}-{}`",
                    bounded_start,
                    bounded_end,
                    file_path.display()
                );
                file_lines.splice(
                    bounded_start..bounded_end,
                    codeblock.code.lines().map(|l| l.to_string()),
                );
            }

            fs::write(&file_path, file_lines.join("\n") + "\n")?;
            info!("Updated: {}", file_path.display());

            Ok(())
        })
        .collect::<Result<(), _>>()?;

    Ok(())
}

fn collect_markdown_files(path: &Path) -> Result<Vec<PathBuf>> {
    if !path.try_exists()? {
        return Err(anyhow!(
            "Path does not exist or is not accessible: {}",
            path.display()
        ));
    }

    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    if !path.is_dir() {
        return Err(anyhow!(
            "Path is neither a file nor a directory: {}",
            path.display()
        ));
    }

    let entries = WalkDir::new(path)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("Failed to read directory: {}", path.display()))?
        .into_iter()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .map(|e| e.into_path())
        .collect();

    Ok(entries)
}

fn handle_preset_result(
    output: &Output,
    preset: &str,
    preset_cfg: &PresetConfig,
    block: &CodeBlock,
    check_only: bool,
) -> anyhow::Result<Option<CodeBlock>> {
    match preset_cfg.output_mode {
        OutputMode::Check => Ok(None),
        OutputMode::Replace => {
            let mismatch = String::from_utf8_lossy(&output.stdout).trim() != block.code.trim();

            if !mismatch {
                debug!(
                    "Skipping code block, content matches output ({})",
                    block.path.display()
                );
                return Ok(None);
            }

            let msg = format!(
                "Code block mismatch detected in `{}:{}-{}` (preset: `{}`, language: `{}`)",
                block.path.display(),
                block.start_line,
                block.end_line,
                preset,
                preset_cfg.language
            );

            if check_only {
                error!("{msg}");
                return Err(anyhow!(msg));
            }

            info!(
                "Code block mismatch will be updated in `{}`",
                block.path.display()
            );

            let updated_code = std::iter::once(format!("```{}", block.headers))
                .chain(
                    String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .lines()
                        .map(|l| l.to_string()),
                )
                .chain(std::iter::once("```".to_string()))
                .map(|l| {
                    format!("{:indent$}{}", "", l, indent = block.indent)
                        .trim_end()
                        .to_string()
                })
                .collect::<Vec<String>>()
                .join("\n");

            Ok(Some(block.with_updated_code(updated_code)))
        }
    }
}
