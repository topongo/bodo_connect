#![allow(dead_code)]

#[cfg(not(feature = "log"))]
use crate::debug;
#[cfg(feature = "log")]
use log::debug;
use std::fmt::{Display, Formatter};

use subprocess::{ExitStatus, Popen, PopenConfig, PopenError, Redirection};

pub trait Process {
    fn get_args(&self) -> Vec<String>;

    fn run(&mut self, opts: Option<PopenConfig>) -> Result<ExitStatus, PopenError> {
        debug!("spawning new ssh process");
        match Popen::create(
            &self.get_args(),
            match opts {
                Some(x) => x,
                None => PopenConfig::default(),
            },
        ) {
            Ok(mut p) => {
                debug!("waiting for ssh process to exit");

                #[cfg(feature = "log")]
                {
                    let o = p.wait();
                    match &o {
                        Ok(e) => debug!("ssh process exited successfully with {:?}", e),
                        Err(e) => debug!("ssh process failed with {:?}", e)
                    }
                    o
                }
                #[cfg(not(feature = "log"))]
                p.wait()
            }
            Err(e) => Err(e),
        }
    }

    fn run_stdout_to_stderr(&mut self) -> Result<ExitStatus, PopenError> {
        debug!("passing redirecting options to ssh process");
        self.run(Some(PopenConfig {
            stdout: Redirection::Merge,
            ..PopenConfig::default()
        }))
    }
}

pub struct SSHProcess {
    args: Vec<String>,
}

impl SSHProcess {
    pub fn new(args: Vec<String>) -> SSHProcess {
        SSHProcess { args }
    }
}

impl Process for SSHProcess {
    fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }
}

impl Display for dyn Process {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_args().join(" "))
    }
}
