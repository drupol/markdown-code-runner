use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum InputMode {
    #[default]
    Stdin,
    File,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OutputMode {
    #[default]
    Replace,
    Check,
}

#[derive(Debug, Deserialize)]
pub struct PresetConfig {
    pub language: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub input_mode: InputMode,
    #[serde(default)]
    pub output_mode: OutputMode,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub presets: HashMap<String, PresetConfig>,
}
