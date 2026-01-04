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
    #[serde(deserialize_with = "deserialize_string_or_vec", alias = "language")]
    pub languages: Vec<String>,
    pub command: Vec<String>,
    #[serde(default)]
    pub input_mode: InputMode,
    #[serde(default)]
    pub output_mode: OutputMode,
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::String(s) => Ok(vec![s]),
        StringOrVec::Vec(v) => Ok(v),
    }
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub presets: HashMap<String, PresetConfig>,
}
