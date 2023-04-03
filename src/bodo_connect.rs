use std::fs::{File, read_to_string};
use log::{debug, error, info, LevelFilter, warn};
use clap::Parser;
use std::future::join;
use std::io::{BufReader, Read};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;
use futures::future::err;
use reqwest::Method;
use subprocess::ExitStatus;


use crate::net::*;
use crate::waker::Waker;
use crate::logger;
use crate::ssh::SSHProcess;

#[derive(Parser, Debug)]
#[command(
    name = "bodoConnect",
    about = "create ssh command on the fly no matter in which network you are"
)]
pub struct BodoConnect {
    #[arg(long, help = "Select different networkmap.json file")]
    networkmap: Option<String>,
    #[arg(short, long, help = "Wake host before connecting")]
    wake: bool,
    #[arg(short, long, help = "Pass -t parameter to ssh (force tty allocation)")]
    tty: bool,
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
    #[arg(short, long, help = "Equivalent to --debug 0")]
    quiet: bool,
    #[arg(short = 'n', long, help = "Send to stdout the generated command without executing it")]
    dry: bool,
    #[arg(short = 'R', long)]
    rsync: bool,
    #[arg(short = 'S', long)]
    sshfs: bool,
    #[arg(short, long)]
    loop_: bool,
    #[arg(short = 'V', long)]
    version: bool,
    host: String,
    extra: Vec<String>
}

pub enum BodoConnectError {
    NoSuchFile(String),
    InvalidPath(String),
    ReadError(String, String),
    ParseError(String),
    DuplicateError(String),
    WakeError(String),
    SSHError(i32),
    SSHUnknownError,
    SpawnError(String),
    NoSuchHost(String)
}

impl BodoConnectError {
    pub fn print_error(&self) {
        match self {
            BodoConnectError::NoSuchFile(p) => error!("no such file or directory: {}", p),
            BodoConnectError::InvalidPath(p) => error!("invalid path: {}", p),
            BodoConnectError::ReadError(f, e) => error!("error reading {}: {}", f, e),
            BodoConnectError::ParseError(e) => error!("parse error: {}", e),
            BodoConnectError::DuplicateError(h) => error!("host duplicate: {}", h),
            BodoConnectError::WakeError(h) => error!("error while waking host {}", h),
            BodoConnectError::SSHError(e) => error!("ssh exited with code {}", e),
            BodoConnectError::SSHUnknownError => error!("unknown ssh error"),
            BodoConnectError::SpawnError(s) => error!("cannot spawn ssh command: {}", s),
            BodoConnectError::NoSuchHost(h) => error!("no such host: {}", h)
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            BodoConnectError::NoSuchFile(..) => 1,
            BodoConnectError::InvalidPath(..) => 2,
            BodoConnectError::ReadError(..) => 3,
            BodoConnectError::ParseError(..) => 4,
            BodoConnectError::DuplicateError(..) => 5,
            BodoConnectError::WakeError(..) => 6,
            BodoConnectError::SSHError(e) => *e,
            BodoConnectError::NoSuchHost(..) => 7,
            BodoConnectError::SSHUnknownError => -1,
            BodoConnectError::SpawnError(..) => -2
        }
    }
}

impl BodoConnect {
    pub fn read_nm_from_file(&self) -> Result<NetworkMap, BodoConnectError> {
        let nm_path = match &self.networkmap {
            Some(p) => {
                let path = Path::new(p);
                if path.exists() && (path.is_file() || path.is_symlink()) {
                    p.clone()
                } else {
                    return Err(BodoConnectError::NoSuchFile(p.clone()))
                }
            }

            None => {
                let p = "~/.config/bodoConnect/networkmap.json".to_string();
                let path = Path::new(&p);
                if !path.exists() {
                    warn!("cannot find networkmap in the default location, using a empty networkmap");
                    return Ok(NetworkMap::new())
                } else { p }
            }
        };

        let subnets: Vec<Subnet> = match serde_json::from_str(
            &*match read_to_string(nm_path.clone()) {
                Ok(s) => s,
                Err(e) => return Err(BodoConnectError::ReadError(e.to_string(), nm_path))
            }
        ) {
            Ok(n) => n,
            Err(e) => return Err(BodoConnectError::ParseError(e.to_string()))
        };

        match NetworkMap::try_from(subnets) {
            Ok(nm) => Ok(nm),
            Err(e) => Err(BodoConnectError::DuplicateError(e))
        }
    }

    pub async fn main(&mut self) -> Result<(), BodoConnectError> {
        log::set_logger(&logger::CONSOLE_LOGGER).unwrap();
        log::set_max_level(
            match self.debug {
                v if v >= 2 => LevelFilter::Debug,
                1 => LevelFilter::Info,
                _ => LevelFilter::Warn
            }
        );

        let mut nm = match self.read_nm_from_file() {
            Ok(n) => n,
            Err(e) => return Err(e)
        };

        // self.static_testing().await

        if let Some(target) = nm.get_host(&self.host) {
            let current_subnet = nm.find_current_subnet().await;
            let a_proc = nm.to_ssh(target, current_subnet, &mut self.extra);

            if self.wake {
                if let Err(s) = nm.wake(target).await {
                    error!("while waking: {}", s);
                }
            }

            let mut proc = a_proc.await;

            if self.dry {
                println!("{}", proc.to_string());
                Ok(())
            } else {
                eprintln!("{}", proc.to_string());

                loop {
                    match match proc.run(None) {
                        Ok(e) => match e {
                            ExitStatus::Exited(s) => {
                                if s == 0 {
                                    Some(Ok(()))
                                } else if self.loop_ {
                                    warn!("ssh exited with {}", s);
                                    None
                                } else {
                                    Some(Err(BodoConnectError::SSHError(s as i32)))
                                }
                            }
                            _ => {
                                Some(Err(BodoConnectError::SSHUnknownError))
                            }
                        }
                        Err(e) => {
                            Some(Err(BodoConnectError::SpawnError(proc.to_string())))
                        }
                    } {
                        Some(r) => return r,
                        None => {}
                    }
                }
            }
        } else {
            Err(BodoConnectError::NoSuchHost(self.host.clone()))
        }
    }

    pub async fn static_testing(&self) {
        let mut nm = NetworkMap::new();
       
        // === redacted ===

        let subnet = match nm.find_current_subnet().await {
            Some(s) => {
                println!("You are in this subnet: {:?}", s);
                Some(s)
            },
            None => {
                println!("can't find known subnet");
                None
            }
        };

        let mut args = Vec::new();
        let a_waker = nm.wake(target);
        let a_proc = nm.to_ssh(target, subnet, &mut args);

        let (waker, mut proc): (Result<(), String>, SSHProcess) = join!(a_waker, a_proc).await;

        if let Err(s) = waker {
            panic!("waker erorr: {}", s)
        }

        eprintln!("{}", proc.get_args().join(" "));
        proc.run(None).expect("ssh error");
    }
}
