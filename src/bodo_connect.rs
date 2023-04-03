use std::fs::read_to_string;
use std::path::Path;
use log::{error, LevelFilter, warn};
use clap::Parser;
use subprocess::ExitStatus;

use crate::net::*;
use crate::logger;
use crate::ssh::SSHOptionStore;
use crate::ssh::options::GenericOption;


#[derive(Parser, Debug)]
#[command(
    name = "bodoConnect",
    about = "create ssh command on the fly no matter in which network you are",
    version = env!("CARGO_PKG_VERSION"),
    author = "topongo"
)]
pub struct BodoConnect {
    #[arg(long, help = "Select different networkmap.json file")]
    networkmap: Option<String>,
    #[arg(short, long, help = "Wake host before connecting")]
    wake: bool,
    #[arg(short, long, help = "Pass -t parameter to ssh (force tty allocation)")]
    tty: bool,
    #[arg(short, action = clap::ArgAction::Count, help = "Set verbosity level")]
    debug: u8,
    #[arg(short, long, help = "Don't log anything")]
    quiet: bool,
    #[arg(short = 'n', long, help = "Send to stdout the generated command without executing it")]
    dry: bool,
    #[arg(short = 'R', long, help = "[WIP] Creates rsync commands")]
    rsync: bool,
    #[arg(short = 'S', long, help = "[WIP] Creates sshfs commands")]
    sshfs: bool,
    #[arg(short, long, help = "Retry connection until ssh returns 0")]
    loop_: bool,
    #[arg(help = "Host to connect to")]
    host: String,
    #[clap(trailing_var_arg=true)]
    #[arg(help = "Extra argument(s), if no -S or -R is used, it will be passed to the remote machine as command")]
    extra: Vec<String>,
}

pub enum BodoConnectError {
    NoSuchFile(String),
    ReadError(String, String),
    ParseError(String),
    DuplicateError(String),
    SSHError(i32),
    SSHUnknownError,
    SpawnError(String, String),
    NoSuchHost(String)
}

impl BodoConnectError {
    pub fn print_error(&self) {
        match self {
            BodoConnectError::NoSuchFile(p) => error!("no such file or directory: {}", p),
            BodoConnectError::ReadError(f, e) => error!("error reading {}: {}", f, e),
            BodoConnectError::ParseError(e) => error!("parse error: {}", e),
            BodoConnectError::DuplicateError(h) => error!("host duplicate: {}", h),
            BodoConnectError::SSHError(e) => error!("ssh exited with code {}", e),
            BodoConnectError::SSHUnknownError => error!("unknown ssh error"),
            BodoConnectError::SpawnError(s, e) => error!("cannot spawn ssh command `{}`: {}", s, e),
            BodoConnectError::NoSuchHost(h) => error!("no such host: {}", h)
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            BodoConnectError::NoSuchFile(..) => 1,
            BodoConnectError::ReadError(..) => 3,
            BodoConnectError::ParseError(..) => 4,
            BodoConnectError::DuplicateError(..) => 5,
            BodoConnectError::SSHError(e) => *e,
            BodoConnectError::NoSuchHost(..) => 7,
            BodoConnectError::SSHUnknownError => -1,
            BodoConnectError::SpawnError(..) => -2
        }
    }
}

impl BodoConnect {
    fn empty() -> Result<NetworkMap, BodoConnectError> {
        warn!("cannot find networkmap in the default location, using a empty networkmap");
        return Ok(NetworkMap::new())
    }

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
                if let Some(home_dir) = home::home_dir() {
                    let p = home_dir.join(".config/bodoConnect/networkmap.json".to_string()).to_str().unwrap().to_string();
                    let path = Path::new(&p);
                    if !path.exists() {
                        return BodoConnect::empty()
                    } else { p }
                } else {
                    return BodoConnect::empty()
                }
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
            if self.quiet {
                LevelFilter::Off
            } else {
                match self.debug {
                    v if v >= 2 => LevelFilter::Debug,
                    1 => LevelFilter::Info,
                    _ => LevelFilter::Warn
                }
            }
        );

        let nm = match self.read_nm_from_file() {
            Ok(n) => n,
            Err(e) => return Err(e)
        };

        // self.static_testing().await

        if let Some(target) = nm.get_host(&self.host) {
            let mut extra_options = SSHOptionStore::new();

            if self.tty {
                extra_options.add_option(Box::new(GenericOption::Switch("-t")))
            }

            let current_subnet = nm.find_current_subnet().await;
            let a_proc = nm.to_ssh(target, current_subnet, &mut self.extra, Some(extra_options));

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
                            Some(Err(BodoConnectError::SpawnError(proc.to_string(), e.to_string())))
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
}
