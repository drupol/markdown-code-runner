use pulldown_cmark::{CodeBlockKind, Event, Parser as MdParser, Tag, TagEnd};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub path: PathBuf,
    pub lang: String,
    pub headers: String,
    pub code: String,
    pub start_line: usize,
    pub end_line: usize,
    pub indent: usize,
}

impl CodeBlock {
    pub fn with_updated_code(&self, new_code: String) -> Self {
        Self {
            code: new_code,
            ..self.clone()
        }
    }
}

pub struct CodeBlockProcessingResult {
    pub replacements: Vec<CodeBlock>,
    pub had_command_failure: bool,
    pub had_mismatch: bool,
}

pub fn parse_code_blocks(path: &Path, content: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut parser = MdParser::new(content).into_offset_iter();

    while let Some((event, range)) = parser.next() {
        if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(headers))) = event {
            if headers.contains("mdcr-skip") {
                // We need to consume until the end of this block
                for (e, _) in &mut parser {
                    if let Event::End(TagEnd::CodeBlock) = e {
                        break;
                    }
                }
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

            for (event, r) in &mut parser {
                match event {
                    Event::Text(text) => {
                        code.push_str(&text);
                        end_offset = r.end;
                    }
                    Event::End(TagEnd::CodeBlock) => {
                        end_offset = r.end;
                        break;
                    }
                    _ => {}
                }
            }

            // Calculate lines
            let start_line = content[..start_offset].lines().count();
            let end_line = content[..end_offset].lines().count();

            // Calculate indentation
            let indent: usize = content
                .get(..start_offset)
                .and_then(|s| s.lines().last())
                .unwrap_or("")
                .chars()
                .take_while(|c| c.is_whitespace())
                .count();

            // Correction for 1-based indexing expectations if any, or just consistent logic
            let start_line = start_line - (indent > 0) as usize;

            blocks.push(CodeBlock {
                path: path.to_path_buf(),
                lang,
                headers: headers.to_string(),
                code,
                start_line,
                end_line,
                indent,
            });
        }
    }
    blocks
}
