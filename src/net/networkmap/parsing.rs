use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::Path;
use crate::net::{NetworkMap, Subnet};

macro_rules! impl_from_nm_parse_error {
    ($variant:path, $ty:path) => {
        impl From<$ty> for NetworkMapParseError {
            fn from(value: $ty) -> Self {
                $variant(value)
            }
        }
    };
}

#[derive(Debug)]
pub struct NotFoundPath {
    inner: String
}

#[derive(Debug)]
pub struct DirPath {
    inner: String
}

impl ToString for NotFoundPath {
    fn to_string(&self) -> String {
        self.inner.clone()
    }
}

impl From<String> for NotFoundPath {
    fn from(value: String) -> Self {
        Self { inner: value }
    }
}

impl ToString for DirPath {
    fn to_string(&self) -> String {
        self.inner.clone()
    }
}

impl From<String> for DirPath {
    fn from(value: String) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug)]
pub enum NetworkMapParseError {
    IOError(std::io::Error),
    SerdeError(serde_json::Error),
    DuplicateError(String),
    FileNotFound(NotFoundPath),
    PathIsDirectory(DirPath)
}

use NetworkMapParseError::*;
impl_from_nm_parse_error!(IOError, std::io::Error);
impl_from_nm_parse_error!(SerdeError, serde_json::Error);
impl_from_nm_parse_error!(DuplicateError, String);
impl_from_nm_parse_error!(FileNotFound, NotFoundPath);
impl_from_nm_parse_error!(PathIsDirectory, DirPath);

impl TryFrom<Vec<Subnet>> for NetworkMap {
    type Error = NetworkMapParseError;

    fn try_from(value: Vec<Subnet>) -> Result<Self, Self::Error> {
        let mut n = NetworkMap::default();
        let mut subs = HashSet::new();
        for s in value.into_iter() {
            if subs.contains(&s.subdomain) {
                return Err(NetworkMapParseError::from(s.subdomain));
            }
            subs.insert(s.subdomain.clone());
            n.add_subnet(s);
        }
        Ok(n)
    }
}

impl TryFrom<String> for NetworkMap {
    type Error = NetworkMapParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let path = Path::new(&value);
        if !path.exists() {
            Err(NetworkMapParseError::from(NotFoundPath::from(value)))
        } else {
            if !path.is_file() {
                Err(NetworkMapParseError::from(DirPath::from(value)))
            } else {
                let subnets: Vec<Subnet> = serde_json::from_str(&read_to_string(value)?)?;

                match NetworkMap::try_from(subnets) {
                    Ok(nm) => Ok(nm),
                    Err(e) => Err(e as NetworkMapParseError),
                }
            }
        }
    }
}