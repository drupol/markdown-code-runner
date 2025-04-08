[![GitHub stars][github stars]][1]
[![Donate!][donate github]][5]

# Markdown-code-runner

A configurable command-line tool that parses Markdown files, extracts code blocks, executes them via external commands, and optionally replaces the blocks with the command output.

Useful for:

- Code formatters (e.g. `black`, `nixfmt`, `shfmt`)
- Linters (e.g. `ruff`, `php -l`, `prettier --check`)
- Auto-updating examples and documentation
- Validating Markdown tutorials with executable code

## Features

- Scans Markdown code blocks by language
- Configurable per-language command execution
- Optional block replacement based on output
- Supports `--check` mode for CI/linting
- Smart placeholder expansion (`{file}`, `{lang}`, etc.)
- Reverse-order processing ensures clean inline replacements
- Configurable with a clean JSON file, validated using [Pydantic](https://docs.pydantic.dev/)

## Installation

```bash
pip install markdown-code-runner
```

Or install locally:

```bash
git clone https://github.com/yourname/markdown-code-runner.git
cd markdown-code-runner
pip install .
```

## Usage

```bash
mdcb path/to/file.md --config config.json
```

### Check Mode (non-destructive)

```bash
mdcb docs/ --check --config config.json
```

This will:

- Execute configured commands for each code block
- Fail with exit code `1` if output differs from original (like a linter)
- Do **not** modify files

## Configuration: `config.json`

The configuration file defines which commands to run for which Markdown block languages.

### Example

```json
{
  "languages": {
    "black-fix": {
      "language": "python",
      "execute": "black {file}",
      "input_mode": "file",
      "replace_output": true
    },
    "ruff-check": {
      "language": "python",
      "execute": "ruff check {file}",
      "input_mode": "file",
      "replace_output": false
    },
    "nixfmt": {
      "language": "nix",
      "execute": "nixfmt < {file}",
      "input_mode": "file",
      "replace_output": true
    }
  }
}
```

## Markdown Syntax

The tool scans for fenced code blocks like:

\`\`\`python
print( "hello" )
\`\`\`

It will execute all matching commands whose `language` is `python`.

## Supported Placeholders

You can use placeholders in the `execute` field:

| Placeholder | Description                      |
| ----------- | -------------------------------- |
| `{file}`    | Path to the temporary code file  |
| `{lang}`    | Language of the block (`python`) |
| `{suffix}`  | File suffix (e.g. `.py`)         |
| `{tmpdir}`  | Temporary directory path         |

## Safeguards

- Blocks with unsupported languages are skipped with a warning.
- `{file}` placeholder is **only allowed** in `input_mode: "file"` mode.
- `replace_output: false` keeps code intact even if the command alters it.

## CI-Friendly `--check` Mode

Use in your CI pipeline:

```bash
mdscan docs/ --check --config config.json
```

It will:

- Run all matching commands
- Return non-zero if the output differs
- Skip rewriting the Markdown file

[github stars]: https://img.shields.io/github/stars/drupol/markdown-code-runner.svg?style=flat-square
[donate github]: https://img.shields.io/badge/Sponsor-Github-brightgreen.svg?style=flat-square
