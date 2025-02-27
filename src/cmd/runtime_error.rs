use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::config::ConfigError;
#[cfg(not(feature = "log"))]
use crate::error;
#[cfg(feature = "log")]
use log::error;
#[cfg(feature = "serde")]
use crate::parse::ParseError;

pub trait Printable: ToString + Debug {}

#[derive(Debug)]
pub enum RuntimeError {
    NoSuchFile(String),
    PathIsDirectory(String),
    IOError(std::io::Error),
    ParseError(String),
    DuplicateError(String),
    #[cfg(feature = "sshfs")]
    TooManyArguments(Vec<String>),
    #[cfg(feature = "sshfs")]
    TooFewArguments,
    SSHError(i32),
    SSHUnknownError,
    SpawnError(String, String),
    NoSuchHost(String),
    MigrationError(Box<RuntimeError>),
    UnknownError(Box<dyn Printable>),
    SerializationError(toml::ser::Error),
    ConfigError(ConfigError),
    #[cfg(feature = "sync")]
    SyncError(String),
    UnknownUnrepresentableError
}

impl RuntimeError {
    #[cfg_attr(not(feature = "log"), allow(unused_variables))]
    pub fn print_error(&self) {
        match self {
            RuntimeError::NoSuchFile(p) => error!("no such file or directory: {}", p),
            RuntimeError::PathIsDirectory(p) => error!("specified path is a directory: {}", p),
            RuntimeError::IOError(e) => error!("error opening/reading file: {}", e),
            RuntimeError::ParseError(e) => error!("parse error: {}", e),
            RuntimeError::DuplicateError(h) => error!("host duplicate: {}", h),
            RuntimeError::SSHError(e) => error!("ssh exited with code {}", e),
            RuntimeError::SSHUnknownError => error!("unknown ssh error"),
            RuntimeError::SpawnError(s, e) => error!("cannot spawn ssh command `{}`: {}", s, e),
            RuntimeError::NoSuchHost(h) => error!("no such host or alias: {}", h),
            RuntimeError::ConfigError(e) => error!("configuration error: {}", e),
            RuntimeError::UnknownError(b) => error!("unkwnown error: {}", b.to_string()),
            RuntimeError::UnknownUnrepresentableError => error!("unkwnown error"),
            #[cfg(feature = "sshfs")]
            RuntimeError::TooManyArguments(a) => error!("extra arguments for sshfs mode: `{}`", a.join(" ")),
            #[cfg(feature = "sshfs")]
            RuntimeError::TooFewArguments => {
                error!("missing parameters for sshfs mode, usage:");
                error!("\tbodoConnect [OPTIONS] --sshfs HOST REMOTE_PATH MOUNT_POINT");
            }
            RuntimeError::MigrationError(e) => {
                error!("there was an error while migrating configuration:");
                e.print_error()
            },
            RuntimeError::SerializationError(e) => {
                error!("serialization error: {}: {:?}", e, e);
            }
            #[cfg(feature = "sync")]
            RuntimeError::SyncError(e) => {
                error!("sync error: {}", e);
            }
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            RuntimeError::NoSuchFile(..) => 1,
            RuntimeError::PathIsDirectory(..) => 8,
            RuntimeError::IOError(..) => 3,
            RuntimeError::ParseError(..) => 4,
            RuntimeError::DuplicateError(..) => 5,
            RuntimeError::SSHError(e) => *e,
            RuntimeError::NoSuchHost(..) => 7,
            RuntimeError::SSHUnknownError => -1,
            RuntimeError::SpawnError(..) => -2,
            RuntimeError::UnknownError(..) => -3,
            RuntimeError::UnknownUnrepresentableError => -4,
            #[cfg(feature = "sshfs")]
            RuntimeError::TooManyArguments(..) => 8,
            #[cfg(feature = "sshfs")]
            RuntimeError::TooFewArguments => 9,
            RuntimeError::MigrationError(..) => 10,
            RuntimeError::SerializationError(..) => 11,
            #[cfg(feature = "sync")]
            RuntimeError::SyncError(..) => 12,
            RuntimeError::ConfigError(..) => 13,
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for RuntimeError {}

impl From<ParseError> for RuntimeError {
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::IOError(e) => RuntimeError::IOError(e),
            ParseError::DuplicateError(s) => RuntimeError::DuplicateError(s),
            ParseError::SerdeJsonError(e) => RuntimeError::ParseError(e.to_string()),
            ParseError::SerdeTomlError(e) => RuntimeError::ParseError(e.to_string()),
            ParseError::SerdeYamlError(e) => RuntimeError::ParseError(e.to_string()),
            ParseError::FileNotFound(p) => RuntimeError::ParseError(p.to_string()),
            ParseError::PathIsDirectory(p) => RuntimeError::PathIsDirectory(p.to_string()),
            ParseError::ConfigError(e) => RuntimeError::ConfigError(e),
        }
    }
}

#[cfg(feature = "serde")]
impl From<toml::ser::Error> for RuntimeError {
    fn from(value: toml::ser::Error) -> Self {
        RuntimeError::SerializationError(value)
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(value: std::io::Error) -> Self {
        RuntimeError::IOError(value)
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for RuntimeError {
    fn from(value: serde_json::Error) -> Self {
        RuntimeError::ParseError(value.to_string())
    }
}

#[cfg(feature = "serde")]
impl From<toml::de::Error> for RuntimeError {
    fn from(value: toml::de::Error) -> Self {
        RuntimeError::ParseError(value.to_string())
    }
}

#[cfg(feature = "serde")]
impl From<serde_yml::Error> for RuntimeError {
    fn from(value: serde_yml::Error) -> Self {
        RuntimeError::ParseError(value.to_string())
    }
}

impl From<ConfigError> for RuntimeError {
    fn from(value: ConfigError) -> Self {
        RuntimeError::ConfigError(value)
    }
}
