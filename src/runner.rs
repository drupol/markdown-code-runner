use crate::command::run_command;
use crate::config::{AppSettings, OutputMode, PresetConfig}; // Added PresetConfig here

use anyhow::anyhow;
use log::{debug, error, info, warn};
use pulldown_cmark::{CodeBlockKind, Event, Parser as MdParser, Tag};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

enum PresetLoopAction {
    Continue,
    Break,
}

type Replacement = (PathBuf, usize, usize, Vec<String>);
type PresetResult = anyhow::Result<(PresetLoopAction, Option<Replacement>)>;

pub struct PresetContext<'a> {
    pub output: &'a str,
    pub preset: &'a str,
    pub preset_cfg: &'a PresetConfig,
    pub path: &'a Path,
    pub code: &'a str,
    pub block_code_headers: &'a str,
    pub original_lines: &'a [&'a str],
    pub start_line: &'a mut usize,
    pub end_line: usize,
    pub check_only: bool,
    pub dry_run: bool,
    pub global_result_mismatch: &'a mut bool,
}

pub fn process(
    path: PathBuf,
    config: &AppSettings,
    check_only: bool,
    dry_run: bool,
) -> Vec<anyhow::Result<()>> {
    collect_markdown_files(&path)
        .par_iter()
        .map(|file| process_markdown_file(file, config, check_only, dry_run))
        .collect::<Vec<anyhow::Result<()>>>()
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
    let mut replacements: Vec<(PathBuf, usize, usize, Vec<String>)> = Vec::new();

    let mut file_had_mismatches = false;
    let file_had_command_failures = false;

    while let Some((event, range)) = parser.next() {
        let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(block_code_headers))) = event else {
            continue;
        };

        let block_code_headers_str = block_code_headers.to_string();
        let mut parts = block_code_headers.split_whitespace();
        let lang = parts.next().unwrap_or_default();

        if block_code_headers.contains("mdcr-skip") {
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

        'preset_loop: for (preset, preset_cfg) in &config.presets {
            if preset_cfg.language.trim() != lang.trim() {
                continue;
            }

            debug!(
                "Processing preset `{}` for language `{}` in mode `{:?}`",
                preset, lang, preset_cfg.mode
            );

            match run_command(preset_cfg, &code) {
                Ok((_, output, _)) => {
                    let mut context = PresetContext {
                        output: &output,
                        preset,
                        preset_cfg,
                        path,
                        code: &code,
                        block_code_headers: &block_code_headers_str,
                        original_lines: &original_lines,
                        start_line: &mut start_line,
                        end_line,
                        check_only,
                        dry_run,
                        global_result_mismatch: &mut file_had_mismatches,
                    };
                    let (action, maybe_replacement) = handle_preset_result(&mut context)?;

                    if let Some(replacement) = maybe_replacement {
                        replacements.push(replacement);
                    }

                    match action {
                        PresetLoopAction::Continue => continue 'preset_loop,
                        PresetLoopAction::Break => break 'preset_loop,
                    }
                }
                Err(e) => {
                    let msg = format!(
                        "Error executing command for preset `{}` in `{}`: {}",
                        preset,
                        path.display(),
                        e
                    );

                    if dry_run {
                        warn!("{}", msg);
                        continue 'preset_loop;
                    }

                    error!("{}", msg);

                    return Err(anyhow!("{}", msg));
                }
            }
        }
    }

    if dry_run {
        return Ok(());
    }

    if check_only {
        if file_had_command_failures {
            return Err(anyhow!(
                "Error(s) while executing commands in file: {}",
                path.display()
            ));
        }

        if file_had_mismatches {
            return Err(anyhow!(
                "Code block mismatch detected in file: {}",
                path.display()
            ));
        }

        debug!("Check mode passed for file: {}", path.display());
        return Ok(());
    }

    if replacements.is_empty() {
        debug!("No changes needed for file: {}", path.display());
        return Ok(()); // No changes, successful processing
    }

    let mut replacements_by_file: HashMap<PathBuf, Vec<(usize, usize, Vec<String>)>> =
        HashMap::new();
    for (file_path, start, end, lines) in replacements {
        replacements_by_file
            .entry(file_path)
            .or_default()
            .push((start, end, lines));
    }

    for (file_path, mut file_replacements) in replacements_by_file {
        let file_content = fs::read_to_string(&file_path)?;
        let mut file_lines: Vec<String> = file_content.lines().map(String::from).collect();

        file_replacements.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));

        for (start, end, new_lines) in file_replacements {
            let bounded_end = std::cmp::min(end, file_lines.len());
            let bounded_start = std::cmp::min(start, bounded_end);

            if bounded_start > file_lines.len() {
                warn!(
                    "Start index {} out of bounds for {}",
                    bounded_start,
                    file_path.display()
                );
                continue;
            }

            debug!(
                "Applying replacement lines {}-{} in {}",
                bounded_start,
                bounded_end,
                file_path.display()
            );
            file_lines.splice(bounded_start..bounded_end, new_lines);
        }

        fs::write(&file_path, file_lines.join("\n") + "\n")?;
        info!("Updated: {}", file_path.display());
    }

    if file_had_command_failures {
        warn!("Updated with command errors: {}", path.display());
    }

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

fn handle_preset_result(ctx: &mut PresetContext) -> PresetResult {
    match ctx.preset_cfg.mode {
        OutputMode::Check => Ok((PresetLoopAction::Continue, None)),
        OutputMode::Replace => {
            let mismatch = ctx.output.trim() != ctx.code.trim();
            *ctx.global_result_mismatch = *ctx.global_result_mismatch || mismatch;

            if !mismatch {
                debug!("Skipping code block, content matches output.");
                return Ok((PresetLoopAction::Continue, None));
            }

            let msg = format!(
                "Code block mismatch detected in: {} (preset: {}, language: {})",
                ctx.path.display(),
                ctx.preset,
                ctx.preset_cfg.language
            );

            if ctx.dry_run {
                warn!("{}", msg);
                return Ok((PresetLoopAction::Continue, None));
            }

            if ctx.check_only {
                return Err(anyhow!("{}", msg));
            }

            info!(
                "Code block mismatch will be updated in: {}",
                ctx.path.display()
            );

            let indent = ctx
                .original_lines
                .get(*ctx.start_line)
                .map(|line| {
                    line.chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>()
                })
                .unwrap_or_default();

            let splice_start_line =
                *ctx.start_line - (!indent.is_empty() && *ctx.start_line > 0) as usize;

            let replacement_lines: Vec<String> =
                std::iter::once(format!("```{}", ctx.block_code_headers))
                    .chain(ctx.output.lines().map(|l| l.to_string()))
                    .chain(std::iter::once("```".to_string()))
                    .map(|l| format!("{}{}", indent, l))
                    .collect();

            Ok((
                PresetLoopAction::Break,
                Some((
                    ctx.path.to_path_buf(),
                    splice_start_line,
                    ctx.end_line,
                    replacement_lines,
                )),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{InputMode, OutputMode, PresetConfig};
    use std::path::PathBuf;

    #[test]
    fn test_handle_preset_result_replacement_generated() {
        let preset = "foo";
        let preset_cfg = PresetConfig {
            language: "sh".into(),
            command: vec!["echo".into(), "Hello".into()],
            input_mode: InputMode::String,
            mode: OutputMode::Replace,
        };

        let output = "Hello\n";
        let code = "echo something";
        let block_code_headers = "sh";
        let path = PathBuf::from("test.md");
        let original_lines = vec!["```sh", "echo something", "```"];
        let mut start_line = 1;
        let end_line = 2;
        let check_only = false;
        let dry_run = false;
        let mut mismatch = false;

        let mut ctx = PresetContext {
            output,
            preset,
            preset_cfg: &preset_cfg,
            path: &path,
            code,
            block_code_headers,
            original_lines: &original_lines,
            start_line: &mut start_line,
            end_line,
            check_only,
            dry_run,
            global_result_mismatch: &mut mismatch,
        };
        let (action, replacement_opt) = handle_preset_result(&mut ctx).unwrap();

        assert!(mismatch);
        assert_eq!(matches!(action, PresetLoopAction::Break), true);
        let (rep_path, rep_start, rep_end, lines) =
            replacement_opt.expect("Expected a replacement");
        assert_eq!(rep_path, path);
        assert_eq!(rep_start, 1);
        assert_eq!(rep_end, end_line);
        assert!(lines.iter().any(|l| l.contains("Hello")));
    }
}
