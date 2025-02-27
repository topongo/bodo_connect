use std::fmt::Display;

use crate::config::ConfigError;

#[derive(Debug)]
pub struct NotFoundPath {
    inner: String
}

#[derive(Debug)]
pub struct DirPath {
    inner: String
}

impl Display for NotFoundPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<String> for NotFoundPath {
    fn from(value: String) -> Self {
        Self { inner: value }
    }
}

impl Display for DirPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<String> for DirPath {
    fn from(value: String) -> Self {
        Self { inner: value }
    }
}

impl From<toml::de::Error> for ParseError {
    fn from(value: toml::de::Error) -> Self {
        ParseError::SerdeTomlError(value)
    }
}

impl From<serde_yml::Error> for ParseError {
    fn from(value: serde_yml::Error) -> Self {
        ParseError::SerdeYamlError(value)
    }
}

#[derive(Debug)]
pub enum ParseError {
    IOError(std::io::Error),
    SerdeJsonError(serde_json::Error),
    SerdeTomlError(toml::de::Error),
    SerdeYamlError(serde_yml::Error),
    DuplicateError(String),
    FileNotFound(NotFoundPath),
    PathIsDirectory(DirPath),
    ConfigError(ConfigError),
}
