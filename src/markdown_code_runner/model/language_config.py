# config.py
from typing import Optional, Literal
from pydantic import BaseModel


class LanguageConfig(BaseModel):
    language: str
    execute: Optional[str] = None
    input_mode: Literal["string", "file"] = "string"
    replace_output: bool = False
