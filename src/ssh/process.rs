#![allow(dead_code)]

#[cfg(not(feature = "log"))]
use crate::debug;
#[cfg(feature = "log")]
use log::debug;
use std::fmt::{Display, Formatter};

use subprocess::{Exec, ExitStatus, PopenError, Redirection};

pub trait Process {
    fn get_args(&self) -> Vec<String>;

    fn exec(&self) -> Exec {
        let args = self.get_args();
        Exec::cmd("ssh")
            // skip first argument of args if it's not empty
            .args(if args.is_empty() { &[] } else { &args[1..] })
    }
    
    fn run(&mut self) -> Result<ExitStatus, PopenError> {
        debug!("spawning new process");
        let exec = self.exec();
        self.inner_run(exec)
    }

    fn inner_run(&self, exec: Exec) -> Result<ExitStatus, PopenError> {
        debug!("waiting for process to exit");
        let r = exec.join();
        #[cfg(feature = "log")]
        match r {
            Ok(ref e) => debug!("process exited successfully with {:?}", e),
            Err(ref e) => debug!("process failed with {:?}", e)
        }
        r
    }

    fn run_stdout_to_stderr(&mut self) -> Result<ExitStatus, PopenError> {
        debug!("passing redirecting options to ssh process");
        let exec = self.exec()
            .stdout(Redirection::Merge);
        self.inner_run(exec)
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
