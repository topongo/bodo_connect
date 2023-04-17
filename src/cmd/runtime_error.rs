use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
#[cfg(not(feature = "log"))]
use crate::error;
#[cfg(feature = "log")]
use log::error;
use crate::net::NetworkMapParseError;

pub trait Printable: ToString + Debug {}

#[derive(Debug)]
pub enum RuntimeError {
    NoSuchFile(String),
    PathIsDirectory(String),
    IOError(String),
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
    UnknownError(Box<dyn Printable>),
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
            RuntimeError::NoSuchHost(h) => error!("no such host: {}", h),
            RuntimeError::UnknownError(b) => error!("unkwnown error: {}", b.to_string()),
            RuntimeError::UnknownUnrepresentableError => error!("unkwnown error"),
            #[cfg(feature = "sshfs")]
            RuntimeError::TooManyArguments(a) => error!("extra arguments for sshfs mode: `{}`", a.join(" ")),
            #[cfg(feature = "sshfs")]
            RuntimeError::TooFewArguments => {
                error!("missing parameters for sshfs mode, usage:");
                error!("\tbodoConnect [OPTIONS] --sshfs HOST REMOTE_PATH MOUNT_POINT");
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
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for RuntimeError {}

impl From<NetworkMapParseError> for RuntimeError {
    fn from(value: NetworkMapParseError) -> Self {
        match value {
            NetworkMapParseError::IOError(e) => RuntimeError::IOError(e.to_string()),
            NetworkMapParseError::DuplicateError(s) => RuntimeError::DuplicateError(s),
            NetworkMapParseError::SerdeError(e) => RuntimeError::ParseError(e.to_string()),
            NetworkMapParseError::FileNotFound(p) => RuntimeError::ParseError(p.to_string()),
            NetworkMapParseError::PathIsDirectory(p) => RuntimeError::PathIsDirectory(p.to_string()),
        }
    }
}
