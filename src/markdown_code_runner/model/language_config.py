# config.py
from typing import Literal
from pydantic import BaseModel


class LanguageConfig(BaseModel):
    language: str
    execute: str
    input_mode: Literal["string", "file"] = "string"
    replace_output: bool = False
