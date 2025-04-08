# config.py
from typing import Dict
from pydantic_settings import BaseSettings, SettingsConfigDict
from .language_config import LanguageConfig
from pathlib import Path


class AppSettings(BaseSettings):
    languages: Dict[str, LanguageConfig]

    model_config = SettingsConfigDict(
        env_prefix="mdcb_",
        env_file=".env",
        yaml_file="config.yaml",  # default fallback
        case_sensitive=False,
    )

    @classmethod
    def from_json_file(cls, path: Path) -> "AppSettings":
        return cls.model_validate_json(path.read_text(encoding="utf-8"))
