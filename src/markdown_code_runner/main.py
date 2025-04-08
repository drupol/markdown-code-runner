import argparse
from pathlib import Path
from markdown_code_runner.model.app_settings import AppSettings
import subprocess
import tempfile
from markdown_it import MarkdownIt
import os


def process_markdown_file(
    filepath: Path, config: AppSettings, check_only: bool = False
) -> bool:
    original_text = filepath.read_text(encoding="utf-8")
    original_lines = original_text.splitlines(keepends=True)

    md = MarkdownIt()
    tokens = md.parse(original_text)

    file_was_updated = False
    changed_detected = False

    for token in reversed(tokens):
        if token.type == "fence" and token.map and token.info:
            lang = token.info.strip()
            line_start, line_end = token.map
            code = token.content

            matching_configs = [
                (name, cfg)
                for name, cfg in config.languages.items()
                if cfg.language == lang
            ]

            if not matching_configs:
                print(f"No config found for language: {lang}")
                continue

            for config_id, lang_config in matching_configs:
                output, changed, should_replace = run_code_block(
                    code, config_id, config
                )

                if changed:
                    changed_detected = True

                if check_only:
                    continue

                if should_replace:
                    output_lines = output.splitlines()
                    replacement_block = (
                        [f"```{lang}{os.linesep}"]
                        + [line + f"{os.linesep}" for line in output_lines]
                        + [f"```{os.linesep}"]
                    )
                    original_lines[line_start:line_end] = replacement_block
                    file_was_updated = True

    if not check_only and file_was_updated:
        filepath.write_text("".join(original_lines), encoding="utf-8")

    return changed_detected


def run_code_block(
    code: str, config_id: str, config: AppSettings
) -> tuple[str, bool, bool]:
    lang_config = config.languages.get(config_id)

    if not lang_config:
        return "", False, False

    output = ""

    if lang_config.input_mode == "string":
        result = subprocess.run(
            lang_config.execute,
            input=code,
            capture_output=True,
            text=True,
            shell=True,
            check=True,
        )

        output = result.stdout.strip()

    elif lang_config.input_mode == "file":
        suffix = f".{lang_config.language}" if lang_config.language else ".tmp"
        with tempfile.NamedTemporaryFile(mode="w+", delete=True, suffix=suffix) as tmp:
            tmp.write(code)
            tmp.flush()

            context = {
                "file": tmp.name,
                "lang": lang_config.language,
                "suffix": suffix,
                "ext": suffix,
                "tmpdir": tempfile.gettempdir(),
            }

            command = lang_config.execute.format(**context)

            result = subprocess.run(
                command,
                capture_output=True,
                text=True,
                shell=True,
                check=True,
            )

        output = result.stdout.strip()

    changed = output.strip() != code.strip()
    should_replace = lang_config.replace_output
    return output, changed, should_replace


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("file", type=str, help="Markdown file to process")
    parser.add_argument(
        "--config", type=str, default="config.json", help="Path to JSON config file"
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Only check if code blocks produce different output",
    )
    args = parser.parse_args()

    config = AppSettings.from_json_file(Path(args.config))
    changed = process_markdown_file(Path(args.file), config, check_only=args.check)

    if args.check:
        if changed:
            print("Code block output mismatch detected.")
            exit(1)
        else:
            print("All code blocks are up-to-date.")
            exit(0)


if __name__ == "__main__":
    main()
