#[cfg(feature = "serde")]
use {
    serde::{Serialize,Deserialize},
    crate::{parse::ParseError,net::Subnet},
    std::fs::read_to_string,
};
use crate::net::NetworkMap;

#[cfg_attr(feature = "serde", derive(Serialize,Deserialize))]
#[derive(Debug,Default)]
pub struct Config {
    pub networkmap: NetworkMap,
    pub settings: Settings,
}

impl Config {
    pub fn split(self) -> (NetworkMap, Settings) {
        (self.networkmap, self.settings)
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
        let content = read_to_string(&value)?;
        if value.ends_with(".yml") || value.ends_with(".yaml") {
            Ok(serde_yml::from_str(&content)?)
        } else if value.ends_with(".toml") {
            Ok(toml::from_str(&content)?)
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

pub static CONFIG_SEARCH_FOLDER: [&'static str; 2] = [
    ".config/bodo_connect",
    ".config/bodoConnect"
];
pub static CONFIG_SEARCH_FILE: [&'static str; 4] = [
    "config.yaml",
    "config.json",
    "config.toml",
    "networkmap.json",
];
