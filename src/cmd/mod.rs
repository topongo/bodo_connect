mod runtime_error;
#[cfg(feature = "sshfs")]
pub mod sshfs;

pub use runtime_error::RuntimeError;

#[cfg(feature = "log")]
use crate::logger::CONSOLE_LOGGER;
#[cfg(not(feature = "log"))]
use crate::{error, warn, info, debug};
#[cfg(feature = "log")]
use log::{error, warn, info, debug, LevelFilter};
use clap::Parser;
use subprocess::ExitStatus;

use crate::net::*;
use crate::ssh::options::GenericOption;
use crate::ssh::SSHOptionStore;


#[derive(Parser, Debug)]
#[command(
    name = "bodoConnect",
    about = "create ssh command on the fly no matter in which network you are",
    version = env!("CARGO_PKG_VERSION"),
    author = "topongo"
)]
pub struct Cmd {
    #[arg(long, help = "Select different networkmap.json file")]
    networkmap: Option<String>,
    #[cfg(feature = "wake")]
    #[arg(short, long, help = "Wake host before connecting")]
    wake: bool,
    #[arg(short, long, help = "Pass -t parameter to ssh (force tty allocation)")]
    tty: bool,
    #[cfg(feature = "log")]
    #[arg(short, action = clap::ArgAction::Count, help = "Set verbosity level")]
    debug: u8,
    #[cfg(feature = "log")]
    #[arg(short, long, help = "Don't log anything")]
    quiet: bool,
    #[arg(
        short = 'n',
        long,
        help = "Send to stdout the generated command without executing it"
    )]
    dry: bool,
    #[cfg(feature = "rsync")]
    #[arg(short = 'R', long, help = "Creates rsync commands")]
    rsync: bool,
    #[cfg(feature = "sshfs")]
    #[arg(short = 'S', long, help = "Creates sshfs commands")]
    sshfs: bool,
    #[arg(short, long, help = "Retry connection until ssh returns 0")]
    loop_: bool,
    #[arg(short = 'e', help = "Specify ssh-like command to execute and eventual options.")]
    cmd: Option<String>,
    #[arg(help = "Host to connect to")]
    host: String,
    #[arg(
        help = "Extra argument(s), if no -S or -R is used, it will be passed to the remote machine as command",
        allow_hyphen_values = true,
        trailing_var_arg = true,
    )]
    extra: Vec<String>
}

impl Cmd {
    fn empty() -> Result<NetworkMap, RuntimeError> {
        warn!("cannot find networkmap in the default location, using a empty networkmap");
        Ok(NetworkMap::default())
    }

    pub fn read_nm_from_file(&self) -> Result<NetworkMap, RuntimeError> {
        match &self.networkmap {
            Some(p) => NetworkMap::try_from(p.clone()).map_err(RuntimeError::from),
            None => {
                info!("networkmap not specified, using the default location");
                if let Some(home_dir) = home::home_dir() {
                    let p = home_dir
                        .join(".config/bodoConnect/networkmap.json")
                        .to_str()
                        .unwrap()
                        .to_string();
                    debug!("default location is: {}", p);
                    NetworkMap::try_from(p).map_err(RuntimeError::from).or_else(|e| {
                        match e {
                            RuntimeError::NoSuchFile(..) => {
                                debug!("not found in `bodoConnect` folde, trying fallback: `bodo_connect`");
                                let p_fallback = home_dir
                                    .join(".config/bodoConnect/networkmap.json")
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                                NetworkMap::try_from(p_fallback).map_err(RuntimeError::from)
                            },
                            _ => Err(e)
                        }
                    })
                } else {
                    error!("cannot get user's home directory");
                    Cmd::empty()
                }
            }
        }

        // debug!("using networkmap file: {}", nm_path);
        //
        // match NetworkMap::try_from(nm_path) {
        //     Ok(n) => Ok(n),
        //     Err(e) => Err(RuntimeError::from(e)),
        // }
    }

    pub async fn main(&mut self) -> Result<(), RuntimeError> {
        #[cfg(feature = "log")]
        {
            log::set_logger(&CONSOLE_LOGGER).unwrap();
            log::set_max_level(if self.quiet {
                LevelFilter::Off
            } else {
                match self.debug {
                    v if v >= 2 => LevelFilter::Debug,
                    1 => LevelFilter::Info,
                    _ => LevelFilter::Warn,
                }
            });
        }

        let nm = match self.read_nm_from_file() {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        if let Some(target) = nm.get_host(&self.host) {
            let mut extra_options = SSHOptionStore::new(self.cmd.clone());

            if self.tty {
                extra_options.add_option(Box::new(GenericOption::Switch("t")))
            }

            debug!("extra arguments: {:?}", self.extra);

            #[cfg(feature = "rsync")]
            if self.rsync {
                debug!("popping elements concerning user and host...");
                if let Some((index, _)) = self.extra
                    .iter()
                    .enumerate()
                    .find(|(_, el)| *el == "-l") {

                    for _ in 0..3 {
                        #[cfg(feature = "log")]
                        {
                            let popped = self.extra.remove(index);
                            debug!("\tpopping: {}", popped);
                        }
                        #[cfg(not(feature = "log"))]
                        self.extra.remove(index);
                    }
                }
            }

            let current_subnet = nm.find_current_subnet().await;

            #[cfg(feature = "sshfs")]
            let mut proc = if self.sshfs {
                if self.extra.len() == 2 {
                    nm.to_sshfs(target, current_subnet, self.extra[0].clone(), self.extra[1].clone()).await
                } else {
                    return if self.extra.len() < 2 {
                        Err(RuntimeError::TooFewArguments)
                    } else {
                        Err(RuntimeError::TooManyArguments(
                            (2..self.extra.len())
                                .map(|e| self.extra[e].clone())
                                .collect::<Vec<String>>()
                        ))
                    }
                }
            } else {
                nm.to_ssh(target, current_subnet, &mut self.extra, Some(extra_options)).await
            };

            #[cfg(not(feature = "sshfs"))]
            let mut proc = nm.to_ssh(target, current_subnet, &mut self.extra, Some(extra_options)).await;
            
            #[cfg(feature = "wake")]
            if self.wake {
                #[cfg(feature = "log")]
                if let Err(s) = nm.wake(target).await {
                    error!("while waking: {}", s);
                }
                #[cfg(not(feature = "log"))]
                if let Err(_) = nm.wake(target).await {
                    // nothing 'till now
                }
            }

            if self.dry {
                println!("{}", proc);
                Ok(())
            } else {
                eprintln!("{}", proc);

                loop {
                    if let Some(r) = match proc.run(None) {
                        Ok(e) => match e {
                            ExitStatus::Exited(s) => {
                                if s == 0 {
                                    Some(Ok(()))
                                } else if self.loop_ {
                                    warn!("ssh exited with {}", s);
                                    None
                                } else {
                                    Some(Err(RuntimeError::SSHError(s as i32)))
                                }
                            }
                            _ => Some(Err(RuntimeError::SSHUnknownError)),
                        },
                        Err(e) => Some(Err(RuntimeError::SpawnError(
                            proc.to_string(),
                            e.to_string(),
                        ))),
                    } {
                        return r;
                    }
                }
            }
        } else {
            Err(RuntimeError::NoSuchHost(self.host.clone()))
        }
    }
}
