use crate::config::ConfigError;

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
