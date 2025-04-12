![GitHub stars][github stars]
[![Donate!][donate github]][5]

# Markdown-code-runner

A configurable command-line tool written in **Rust** that parses Markdown files, extracts fenced code blocks, executes them via external arbitrary commands, and optionally replaces the content of the blocks with the command output.

Useful for:

- Validating Markdown tutorials with executable code
- Auto-updating examples and documentation
- Code formatters (e.g. `black`, `nixfmt`, `shfmt`)
- Linters (e.g. `ruff`, `php -l`, `prettier --check`)

## Features

- Clean configuration via a TOML file
- Fast and dependency-free thanks to Rust
- Scans fenced Markdown code blocks by language
- Configurable per-language command execution
- Optional block replacement based on command output
- `--check` mode for CI/linting use cases
- Markdown code blocks can opt-out using the `mdcr-skip` flag
- Placeholder support (`{file}`, `{lang}`, etc.)

## Installation

```bash
git clone https://github.com/drupol/markdown-code-runner
cd markdown-code-runner
cargo build --release
./target/release/mdcr --help
```

### Via nix

Available soon through `markdown-code-runner` package, the binary is called `mdcr`.

## Usage

```bash
mdcr path/to/file.md --config config.toml
```

### Check Mode (non-destructive)

```bash
mdcr docs/ --check --config config.toml
```

This will:

- Execute configured commands for each code block
- Fail with exit code `1` if output differs from original (like a linter)
- Do **not** modify files

## Configuration: `config.toml`

The configuration file defines which commands to run for which Markdown block languages.

### Example

Save this file as `config.toml`:

```toml
[presets.ruff-format]
language = "python"
command = ["ruff", "format", "-"]
input_mode = "stdin"
replace_output = true

[presets.nixfmt]
language = "nix"
command = ["nixfmt"]
input_mode = "stdin"
replace_output = true

[presets.php]
language = "php"
command = [
  "sh",
  "-c",
  "php-cs-fixer fix -q --rules=@PSR12 {file}; cat {file}"
]
input_mode = "file"
mode = "replace"

[presets.rust]
language = "rust"
command = ["rustfmt"]
input_mode = "stdin"
replace_output = true

[presets.typstyle]
language = "typst"
command = ["typstyle"]
input_mode = "stdin"
replace_output = true

[presets.latex]
language = "latex"
command = ["tex-fmt", "--stdin"]
input_mode = "stdin"
replace_output = true
```

Then run it into a directory:

```sh
mdcr --config config.toml /path/to/doc/
```

## Markdown Syntax

The tool scans for fenced code blocks like:

````
```python
print( "hello" )
```
````

It will execute all matching commands whose `language` is `python`.

### Skipping a code block

To exclude a block from processing, add mdcr-skip after the language:

````
```python mdcr-skip
print("don't touch this")
```
````

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
- `{file}` placeholder is **only available** in `input_mode: "file"` mode.

## CI-Friendly `--check` Mode

Use in your CI pipeline:

```bash
mdcr docs/ --check --config config.toml
```

It will:

- Run all matching commands
- Return non-zero if the output differs or if any command fails
- Skip rewriting the Markdown file

[github stars]: https://img.shields.io/github/stars/drupol/markdown-code-runner.svg?style=flat-square
[donate github]: https://img.shields.io/badge/Sponsor-Github-brightgreen.svg?style=flat-square
[5]: https://github.com/sponsors/drupol
