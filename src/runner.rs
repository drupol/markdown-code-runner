use crate::config::{AppSettings, OutputMode, PresetConfig};

use crate::command::{command_to_string, run_command};
use anyhow::anyhow;
use anyhow::{Context, Result};
use log::{debug, error, info};
use pulldown_cmark::{CodeBlockKind, Event, OffsetIter, Parser as MdParser, Tag};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::rc::Rc;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub path: PathBuf,
    pub lang: String,
    pub headers: String,
    pub code: String,
    pub start_line: usize,
    pub end_line: usize,
    pub indent: String,
}

impl CodeBlock {
    pub fn with_updated_code(&self, new_code: String) -> Self {
        Self {
            code: new_code,
            ..self.clone()
        }
    }
}

pub struct CodeBlockIterator {
    path: PathBuf,
    content: Rc<str>,
    parser: OffsetIter<'static>,
}

struct BlockProcessingResult {
    replacements: Vec<CodeBlock>,
    had_command_failure: bool,
    had_mismatch: bool,
}

impl CodeBlockIterator {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let content: Rc<str> = Rc::from(fs::read_to_string(path)?);

        let content_static: &'static str =
            unsafe { std::mem::transmute::<&str, &'static str>(content.as_ref()) };

        let parser = MdParser::new(content_static).into_offset_iter();

        Ok(CodeBlockIterator {
            path: path.to_path_buf(),
            content,
            parser,
        })
    }
}

impl Iterator for CodeBlockIterator {
    type Item = CodeBlock;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((event, range)) = self.parser.next() {
            let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(headers))) = event else {
                continue;
            };

            if headers.contains("mdcr-skip") {
                continue;
            }

            let lang = headers
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string();

            let mut code = String::new();
            let start_offset = range.start;
            let mut end_offset = range.end;

            for (event, r) in &mut self.parser {
                match event {
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

            let content_str = self.content.as_ref();
            let start_line = content_str[..start_offset].lines().count();
            let end_line = content_str[..end_offset].lines().count();

            let indent: String = content_str
                .get(..start_offset)
                .and_then(|s| s.lines().last())
                .unwrap_or("")
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();

            let start_line = start_line - (!indent.is_empty() && start_line > 0) as usize;

            return Some(CodeBlock {
                path: self.path.clone(),
                lang,
                headers: headers.to_string(),
                code,
                start_line,
                end_line,
                indent,
            });
        }

        None
    }
}

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

    let results: Vec<BlockProcessingResult> = blocks
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
) -> BlockProcessingResult {
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

    BlockProcessingResult {
        replacements,
        had_command_failure,
        had_mismatch,
    }
}

fn apply_replacements(replacements: Vec<CodeBlock>) -> Result<()> {
    let mut replacements_by_file: HashMap<PathBuf, Vec<(usize, usize, Vec<String>)>> =
        HashMap::new();

    for block in replacements {
        replacements_by_file
            .entry(block.path.clone())
            .or_default()
            .push((
                block.start_line,
                block.end_line,
                block.code.lines().map(String::from).collect(),
            ));
    }

    replacements_by_file
        .into_par_iter()
        .map(|(file_path, mut file_replacements)| {
            let file_content = fs::read_to_string(&file_path)?;
            let mut file_lines: Vec<String> = file_content.lines().map(String::from).collect();

            file_replacements.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));

            for (start, end, new_lines) in file_replacements {
                let bounded_end = end.min(file_lines.len());
                let bounded_start = start.min(bounded_end);
                debug!(
                    "Applying replacement lines `{}:{}-{}`",
                    bounded_start,
                    bounded_end,
                    file_path.display()
                );
                file_lines.splice(bounded_start..bounded_end, new_lines);
            }

            fs::write(&file_path, file_lines.join("\n") + "\n")?;
            info!("Updated: {}", file_path.display());

            Ok::<_, anyhow::Error>(())
        })
        .collect::<Result<Vec<_>, _>>()?;

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
                error!("{}", msg);
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
                .map(|l| format!("{}{}", block.indent, l))
                .collect::<Vec<String>>()
                .join("\n");

            Ok(Some(block.with_updated_code(updated_code)))
        }
    }
}
