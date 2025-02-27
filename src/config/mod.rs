#[cfg(feature = "serde")]
use {
    serde::{Serialize,Deserialize},
    crate::{parse::ParseError,net::Subnet},
    std::fs::read_to_string,
};
use crate::net::{NetworkMap, NetworkMapError};
use std::{path::PathBuf, fmt::Display};

#[cfg_attr(feature = "serde", derive(Serialize,Deserialize))]
#[derive(Debug,Default)]
pub struct Config {
    pub networkmap: NetworkMap,
    pub settings: Settings,
}

#[derive(Debug)]
pub enum ConfigError {
    NetworkMap(NetworkMapError),
    // Settings(SettingsError),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NetworkMap(e) => write!(f, "networkmap error: {}", e),
            // ConfigError::Settings(e) => write!(f, "SettingsError: {}", e),
        }
    }
}

impl From<NetworkMapError> for ConfigError {
    fn from(value: NetworkMapError) -> Self {
        ConfigError::NetworkMap(value)
    }
}

impl Config {
    pub fn split(self) -> (NetworkMap, Settings) {
        (self.networkmap, self.settings)
    }

    pub fn default_path(base: Option<PathBuf>) -> PathBuf {
        base
            .unwrap_or_default()
            .join(CONFIG_SEARCH_FOLDER[0])
            .join(CONFIG_SEARCH_FILE[0])
    }

    pub fn check(&self) -> Result<(), ConfigError> {
        self.networkmap.check()?;
        // self.settings.check()?;
        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize,Deserialize))]
#[derive(Debug,Default)]
pub struct Settings {
}

#[cfg(feature = "serde")]
impl<'a> TryFrom<&'a str> for Config {
    type Error = ParseError; 

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let content = read_to_string(value)?;
        if value.ends_with(".yml") || value.ends_with(".yaml") {
            let c: Self = serde_yml::from_str(&content)?;
            c.check()?;
            Ok(c)
        } else if value.ends_with(".toml") {
            let c: Self = toml::from_str(&content)?;
            c.check()?;
            Ok(c)
        } else {
            match serde_json::from_str(&content) {
                Ok(c) => Ok(c),
                Err(e) => {
                    // fallback to old networkmap format
                    match serde_json::from_str::<Vec<Subnet>>(&content) {
                        Ok(nm) => Ok(Config {
                            networkmap: NetworkMap::try_from(nm)?,
                            settings: Settings::default(),
                        }),
                        Err(_) => Err(ParseError::SerdeJsonError(e))
                    }
                }
            }
        }
    }
}

#[cfg(feature = "serde")]
impl TryFrom<String> for Config {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

pub static CONFIG_SEARCH_FOLDER: [&str; 2] = [
    ".config/bodo_connect",
    ".config/bodoConnect"
];
pub static CONFIG_SEARCH_FILE: [&str; 4] = [
    "config.yaml",
    "config.json",
    "config.toml",
    "networkmap.json",
];
