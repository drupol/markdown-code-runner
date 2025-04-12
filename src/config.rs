use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    Stdin,
    File,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Replace,
    Check,
}

#[derive(Debug, Deserialize)]
pub struct PresetConfig {
    pub language: String,
    pub command: Vec<String>,
    pub input_mode: InputMode,
    pub mode: OutputMode,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub presets: HashMap<String, PresetConfig>,
}
