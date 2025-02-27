use std::fs::read_to_string;
use std::path::Path;
use crate::config::ConfigError;
use crate::net::{NetworkMap, Subnet};
use crate::parse::{DirPath, NotFoundPath, ParseError};

macro_rules! impl_from_parse_error {
    ($variant:path, $ty:path) => {
        impl From<$ty> for ParseError {
            fn from(value: $ty) -> Self {
                $variant(value)
            }
        }
    };
}

use ParseError::*;

use super::NetworkMapError;
impl_from_parse_error!(IOError, std::io::Error);
impl_from_parse_error!(SerdeJsonError, serde_json::Error);
impl_from_parse_error!(DuplicateError, String);
impl_from_parse_error!(FileNotFound, NotFoundPath);
impl_from_parse_error!(PathIsDirectory, DirPath);
impl_from_parse_error!(ConfigError, ConfigError);

impl From<NetworkMapError> for ParseError {
    fn from(value: NetworkMapError) -> Self {
        ConfigError(value.into())
    }
}

impl TryFrom<Vec<Subnet>> for NetworkMap {
    type Error = NetworkMapError;

    fn try_from(value: Vec<Subnet>) -> Result<Self, Self::Error> {
        let mut n = NetworkMap::default();
        for s in value {
            n.add_subnet(s);
        } 
        n.check()?;
        Ok(n)
    }
}

impl<'a> TryFrom<&'a str> for NetworkMap {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let path = Path::new(&value);
        if !path.exists() {
            Err(ParseError::from(NotFoundPath::from(value.to_owned())))
        } else if !path.is_file() {
                Err(ParseError::from(DirPath::from(value.to_owned())))
        } else {
            let content = read_to_string(value)?;
            let subnets: Vec<Subnet> = match toml::from_str(&content) {
                Ok(s) => s,
                Err(_) => serde_json::from_str(&content)?,
            };

            Ok(NetworkMap::try_from(subnets)?)
        }
    }
}

impl TryFrom<String> for NetworkMap {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
