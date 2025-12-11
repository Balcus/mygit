use std::fs;
use std::fs::File;
use std::path::PathBuf;
use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfigFields {
    user_name: Option<String>,
    user_email: Option<String>
}

pub struct Config {
    path: PathBuf,
    user_name: Option<String>,
    user_email: Option<String>
}

impl Config {
    pub fn default(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();

        File::create(&path)
            .with_context(|| format!("Cannot initialize config file at {:?}", path))?;

        Ok(Self {
            path,
            user_name: None,
            user_email: None,
        })
    }

    pub fn from(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Could not read config file {:?}", path))?;

        let fields: ConfigFields = toml::from_str(&content)
            .with_context(|| "Failed parsing config file")?;

        Ok(Self {
            path,
            user_name: fields.user_name,
            user_email: fields.user_email
        })
    }
}
