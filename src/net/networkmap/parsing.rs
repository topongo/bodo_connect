use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::Path;
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
impl_from_parse_error!(IOError, std::io::Error);
impl_from_parse_error!(SerdeJsonError, serde_json::Error);
impl_from_parse_error!(DuplicateError, String);
impl_from_parse_error!(FileNotFound, NotFoundPath);
impl_from_parse_error!(PathIsDirectory, DirPath);

impl TryFrom<Vec<Subnet>> for NetworkMap {
    type Error = ParseError;

    fn try_from(value: Vec<Subnet>) -> Result<Self, Self::Error> {
        let mut n = NetworkMap::default();
        let mut subs = HashSet::new();
        for s in value.into_iter() {
            if subs.contains(&s.subdomain) {
                return Err(ParseError::from(s.subdomain));
            }
            subs.insert(s.subdomain.clone());
            n.add_subnet(s);
        }
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

            match NetworkMap::try_from(subnets) {
                Ok(nm) => Ok(nm),
                Err(e) => Err(e as ParseError),
            }
        }
    }
}

impl TryFrom<String> for NetworkMap {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
