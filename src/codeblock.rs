use pulldown_cmark_codeblock::{CodeBlock as MarkdownCodeBlock, code_blocks};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub path: PathBuf,
    pub markdown: MarkdownCodeBlock,
    pub replacement: Option<String>,
}

impl CodeBlock {
    pub fn with_updated_code(&self, new_code: String) -> Self {
        Self {
            replacement: Some(new_code),
            ..self.clone()
        }
    }

    pub fn language(&self) -> &str {
        self.markdown.language.as_deref().unwrap_or_default()
    }

    pub fn source(&self) -> &str {
        &self.markdown.source
    }

    pub fn replacement_source(&self) -> &str {
        self.replacement.as_deref().unwrap_or(&self.markdown.source)
    }

    pub fn info_string(&self) -> &str {
        &self.markdown.info_string
    }

    pub fn start_line(&self) -> usize {
        self.markdown.line_range.start
    }

    pub fn end_line(&self) -> usize {
        self.markdown.line_range.end
    }

    pub fn indent(&self) -> usize {
        self.markdown.indent
    }
}

pub struct CodeBlockProcessingResult {
    pub replacements: Vec<CodeBlock>,
    pub had_command_failure: bool,
    pub had_mismatch: bool,
}

pub fn parse_code_blocks(path: &Path, content: &str) -> Vec<CodeBlock> {
    code_blocks(content)
        .filter(|block| block.is_fenced())
        .filter(|block| !block.has_info_word("mdcr-skip"))
        .map(|block| CodeBlock {
            path: path.to_path_buf(),
            markdown: block,
            replacement: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_code_blocks;
    use std::path::Path;

    #[test]
    fn mdcr_parser_filters_skip_blocks() {
        let markdown = "```rust mdcr-skip\nignored\n```\n\n```rust\nkept\n```\n";

        let blocks = parse_code_blocks(Path::new("test.md"), markdown);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language(), "rust");
        assert_eq!(blocks[0].source(), "kept\n");
    }
}
