pub mod hop;
mod options_internal;
pub mod process;

pub mod options {
    use crate::ssh::options_internal;
    pub use options_internal::{GenericOption, JumpHosts, PortOption};
}

pub use hop::Hop;
#[cfg(feature = "log")]
pub use hop::join_hops;
pub use options_internal::SSHOptionStore;
pub use process::SSHProcess;

pub const fn default_port() -> u16 {
    22
}
