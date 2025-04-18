use pulldown_cmark::{CodeBlockKind, Event, OffsetIter, Parser as MdParser, Tag};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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

pub struct CodeBlockIterator {
    path: PathBuf,
    content: Rc<str>,
    parser: OffsetIter<'static>,
}

pub struct CodeBlockProcessingResult {
    pub replacements: Vec<CodeBlock>,
    pub had_command_failure: bool,
    pub had_mismatch: bool,
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

            let indent: usize = content_str
                .get(..start_offset)
                .and_then(|s| s.lines().last())
                .unwrap_or("")
                .chars()
                .take_while(|c| c.is_whitespace())
                .count();

            let start_line = start_line - (indent > 0) as usize;

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
