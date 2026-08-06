![GitHub stars][GitHub stars]
[![Crates.io Version][Crates.io Version]][mdcr crates]
[![Crates.io License][Crates.io License]][mdcr crates]
[![Donate!][Donate!]][sponsor link]

# MDCR - Markdown Code Runner

A configurable command-line tool written in **Rust** that parses Markdown files, extracts fenced code blocks, executes them via external arbitrary commands, and optionally replaces the content of the blocks with the command output.

Useful for:

- Validating Markdown tutorials with executable code
- Auto-updating examples and documentation
- Code formatting code blocks (e.g. `black`, `nixfmt`, `shfmt`, `php-cs-fixer`, etc)
- Linting code blocks (e.g. `ruff`, `php -l`, `prettier`, etc)

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

### Via Cargo

You can install the binary with Cargo:

```sh
cargo install mdcr
```

### Via Nixpkgs

Available via the [`markdown-code-runner` package], the binary is called `mdcr`.

### Via the source code

Clone the repository and run in the sourcecode folder:

```sh
cargo build --release
```

The binary will be in `target/release/mdcr`.

### Via Nix

You can use the package from this repository with Nix. If you have Nix installed, you can run the tool directly:

```sh
nix run github:drupol/markdown-code-runner
```

## Usage

```sh
mdcr --config config.toml path/to/file.md
```

### Check Mode (non-destructive)

```sh
mdcr --config config.toml --check path/to/file.md
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
languages = ["python", "py"]
command = ["ruff", "format", "-"]

[presets.nixfmt]
language = "nix"
command = ["nixfmt"]

[presets.php]
language = "php"
# php-cs-fixer does not support STDIN, so we use a temporary file
command = [
  "sh",
  "-c",
  "php-cs-fixer fix -q --rules=@PSR12 {file}; cat {file}"
]
input_mode = "file"

[presets.rust]
language = "rust"
command = ["rustfmt"]

[presets.typstyle]
language = "typst"
command = ["typstyle"]

[presets.latex]
language = "latex"
command = ["tex-fmt", "--stdin"]
```

#### Input Modes

Each preset supports an optional `input_mode`, which defines how the code block is passed to the command:

- `stdin` (default): The code is passed via standard input (`STDIN`)
- `file`: The code is written to a temporary file and its path is passed, the temporary file is deleted immediately after execution

#### Output Modes

Each preset also supports an optional `output_mode`, which defines how the command output is used:

- `replace` (default): Replace the code block content with the command's output
- `check`: Check the command's exit code, if it is different from `0`, the command failed, and the tool will return a non-zero exit code

If not specified, both `input_mode` and `output_mode` default to `stdin` and `replace`, respectively.

#### Updating the output block language

Presets in `replace` mode can also update the Markdown code block language by setting `output_language`:

```toml
[presets.nixtojson]
language = "nixToJson"
command = ["nix", "eval", "--json", "--file", "{file}"]
input_mode = "file"
output_mode = "replace"
output_language = "json"
```

This lets a generated block change from:

````
```nixToJson
{ enabled = true; }
```
````

to:

````
```json
{"enabled":true}
```
````

## Markdown Syntax

The tool scans for fenced code blocks like:

````
```python
print( "hello" )
```
````

It will execute all matching commands whose `language` is `python`.

### Skipping a code block

To exclude a block from processing, add `mdcr-skip` after the language:

````
```python mdcr-skip
print("don't touch this")
```
````

## Supported Placeholders

You can use placeholders in the `command` field:

| Placeholder | Description                      |
| ----------- | -------------------------------- |
| `{file}`    | Path to the temporary code file  |
| `{lang}`    | Language of the block (`python`) |
| `{suffix}`  | File suffix (e.g. `.py`)         |
| `{tmpdir}`  | Temporary directory path         |

## Safeguards

- Blocks with unsupported languages are skipped with a warning.
- `{file}` placeholder is **only available** in `input_mode: "file"` mode.

## CI Integration

Recommended usage in continuous integration:

```bash
mdcr --config config.toml --check docs/
```

This runs all configured checks and returns a non-zero exit code if:

- Output differs from the original
- A command fails

The `--check` mode will not modify any files.

## Logging

The CLI option `--log` allows you to control the verbosity and destination of log messages emitted during execution.

### Usage

```bash
mdcr --config config.toml --log debug path/to/file.md
```

### Available log levels

The logging system uses standard log levels, from most verbose to least:

| Level   | Description                                           |
| ------- | ----------------------------------------------------- |
| `trace` | Highly detailed, useful for debugging internal issues |
| `debug` | General debugging information                         |
| `info`  | Informational messages about execution progress       |
| `warn`  | Non-critical issues that deserve attention            |
| `error` | Critical problems encountered during execution        |

By default, if no `--log` option is provided, the logging level defaults to `warn`.

[GitHub stars]: https://img.shields.io/github/stars/drupol/markdown-code-runner.svg?style=flat-square
[Donate!]: https://img.shields.io/badge/Sponsor-Github-brightgreen.svg?style=flat-square
[sponsor link]: https://github.com/sponsors/drupol
[Crates.io License]: https://img.shields.io/crates/l/mdcr?style=flat-square
[Crates.io Version]: https://img.shields.io/crates/v/mdcr?style=flat-square
[mdcr crates]: https://crates.io/crates/mdcr
[`markdown-code-runner` package]: https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=markdown-code-runner
