import json
from pathlib import Path
import subprocess

import pytest


@pytest.fixture
def sample_markdown_file(tmp_path: Path) -> tuple[Path, Path]:
    markdown_file = tmp_path / "test.md"
    markdown_file.write_text("""\
```sh
echo "hello"
""")

    config_file = tmp_path / "config.json"
    config = {
        "languages": {
            "shell-echo": {
                "language": "sh",
                "execute": "echo hello",
                "input_mode": "string",
                "replace_output": True,
            }
        }
    }
    config_file.write_text(json.dumps(config))

    return markdown_file, config_file


def test_main_check_code_block(sample_markdown_file: tuple[Path, Path]):
    markdown_file, config_file = sample_markdown_file

    result = subprocess.run(
        [
            "python",
            "-m",
            "markdown_code_runner.main",
            str(markdown_file),
            "--check",
            "--config",
            str(config_file),
        ],
        capture_output=True,
        text=True,
    )

    assert result.returncode == 1


def test_main_rewrites_code_block(sample_markdown_file: tuple[Path, Path]):
    markdown_file, config_file = sample_markdown_file

    result = subprocess.run(
        [
            "python",
            "-m",
            "markdown_code_runner.main",  # Adjust if the entry point is elsewhere
            str(markdown_file),
            "--config",
            str(config_file),
        ],
        capture_output=True,
        text=True,
        check=True,
    )

    assert result.returncode == 0
    updated = markdown_file.read_text()

    assert updated.strip() == """```sh\nhello\n```"""
