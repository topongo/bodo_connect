mod runtime_error;
#[cfg(feature = "sshfs")]
pub mod sshfs;

use std::path::PathBuf;
use std::time::{Duration, Instant};

pub use runtime_error::RuntimeError;

use crate::config::Config;
#[cfg(feature = "log")]
use crate::logger::CONSOLE_LOGGER;
#[allow(unused_imports)]
#[cfg(not(feature = "log"))]
use crate::{error, warn, info, debug};
#[allow(unused_imports)]
#[cfg(feature = "log")]
use log::{error, warn, info, debug, LevelFilter};
use clap::{Parser,CommandFactory};
use clap::error::{ContextKind, ContextValue, ErrorKind, RichFormatter};
use subprocess::ExitStatus;

use crate::ssh::options::GenericOption;
use crate::ssh::SSHOptionStore;
use crate::config::{CONFIG_SEARCH_FILE,CONFIG_SEARCH_FOLDER};
use std::io::Write;

#[derive(Parser, Debug)]
#[command(
    name = "bodoConnect",
    about = "create ssh command on the fly no matter in which network you are",
    version = env!("CARGO_PKG_VERSION"),
    author = "topongo"
)]
pub struct Cmd {
    #[arg(long, help = "Select different config file")]
    config: Option<String>,
    #[cfg(feature = "wake")]
    #[arg(short, long, help = "Wake host before connecting")]
    wake: bool,
    #[arg(short, long, help = "Pass -t parameter to ssh (force tty allocation)")]
    tty: bool,
    #[cfg(feature = "log")]
    #[arg(short, action = clap::ArgAction::Count, help = "Set verbosity level")]
    debug: u8,
    #[arg(short, long, help = "Don't log anything, suppress generated command echoing")]
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
    #[arg(long, help = "Migrate from json to yaml format")]
    pub migrate_to_yaml: bool,
    #[cfg(feature = "sync")]
    #[arg(long, help = "Push loaded configuration with host marked as `sync`. Implies -n")]
    pub push_config: bool,
    #[cfg(feature = "sync")]
    #[arg(long, help = "Pull configuration to disk. Overwrites existing config! Argument --config will be considered")]
    pub pull_config: bool,
    #[arg(help = "Host to connect to")]
    host: Option<String>,
    #[arg(
        help = "Extra argument(s), if no -S or -R is used, it will be passed to the remote machine as command",
        allow_hyphen_values = true,
        trailing_var_arg = true,
    )]
    extra: Vec<String>
}

impl Cmd {
    pub fn search_cfg(&self) -> Vec<String> {
        let home_dir = match home::home_dir() {
            Some(h) => h,
            None => {
                match users::get_current_username() {
                    Some(u) => {
                        PathBuf::from("/home").join(u)
                    }
                    None => {
                        return vec![];
                    }
                }
            }
        };

        let mut results = vec![];
        for i in CONFIG_SEARCH_FOLDER
            .iter()
            .map(|f| home_dir.join(f)) 
        {
            if i.exists() {
                for j in CONFIG_SEARCH_FILE.iter() {
                    let p = i.join(j);
                    if p.exists() {
                        results.push(p.to_str().unwrap().to_owned());
                    }
                }
            }
        }

        let default = Config::default_path(Some(home_dir));
        if results.is_empty() && default != PathBuf::from(&results[0]) {
            warn!(
                "deprecation warning: configuration is not in the default location ({:?})",
                default
            )
        }
        results
    }

    pub fn load_cfg(&self) -> Result<Config, RuntimeError> {
        match &self.config {
            Some(f) => self.cfg_from_file(f),
            None => {
                info!("config not specified, searching default locations");
                let nms = self.search_cfg();
                debug!("found configurations: {:?}", nms);
                if nms.is_empty() {
                    Err(RuntimeError::ParseError("no networkmap file found.".to_owned()))
                } else {
                    debug!("selecting configuration: {:?}", nms[0]);
                    self.cfg_from_file(&nms[0])
                }
            }
        }
    }

    pub fn cfg_from_file(&self, file: &str) -> Result<Config, RuntimeError> {
        Config::try_from(file).map_err(RuntimeError::from)
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

        if self.migrate_to_yaml {
            return self.migrate_config()
        }

        #[cfg(feature = "sync")]
        if self.pull_config || self.push_config {
            if self.pull_config && self.push_config {
                return Err(RuntimeError::SyncError("pull and push cannot be used together".to_owned()));
            } else {
                return self.sync_config().await;
            }
        }

        let cfg = match self.load_cfg() {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        // settings don't exist yet
        let (nm, _settings) = cfg.split();

        if let Some(target) = nm.get_host(&self.host.clone().unwrap()) {
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
                nm.to_ssh(target, current_subnet, &self.extra, Some(extra_options)).await
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
                if !self.quiet {
                    println!("{}", proc);
                }
                Ok(())
            } else {
                if !self.quiet {
                    eprintln!("{}", proc);
                }

                let mut connection_start;
                loop {
                    connection_start = Instant::now();
                    if let Some(r) = match proc.run() {
                        Ok(e) => match e {
                            ExitStatus::Exited(s) => {
                                if s == 0 {
                                    Some(Ok(()))
                                } else if self.loop_ {
                                    if s == 255 {
                                        warn!("ssh exited with {}", s);
                                        None
                                    } else {
                                        Some(Err(RuntimeError::SSHError(s as i32)))
                                    }
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
                    } else if connection_start.elapsed().as_millis() < 200 {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        } else {
            Err(RuntimeError::NoSuchHost(self.host.clone().unwrap()))
        }
    }

    pub fn check_host(&self) {
        if self.host.is_none() {
            let mut c = Self::command();
            let mut err = clap::error::Error::<RichFormatter>::new(ErrorKind::MissingRequiredArgument)
                .with_cmd(&c);

            err.insert(ContextKind::InvalidArg, ContextValue::Strings(vec!["<HOST>".to_owned()]));
            err.insert(ContextKind::Usage, ContextValue::StyledStr(c.render_usage()));
            err.exit();
        }
    }

    pub fn migrate_config(&mut self) -> Result<(), RuntimeError> {
        debug!("migrating config, json -> yaml");
        debug!("searching for all configuration");
        let nms = self.search_cfg();
        for i in &nms {
            if i.ends_with(".yaml") {
                info!("found yaml configuration: {:?}", i);
                warn!("a yaml configuration already exists. exiting.");
                return Ok(())
            }
        }
        
        debug!("configurations found: {:?}", nms);
        if let Some(f) = nms.iter().filter(|f| f.ends_with(".json")).nth(0) {
            let cfg = Config::try_from(f.as_str())?;
            let p_out = PathBuf::from(f);
            let p_out = p_out.parent().unwrap();
            let f_out = p_out.join("config.yaml");
            let mut output = std::fs::File::create(&f_out)?;
            write!(output, "{}", serde_yml::to_string(&cfg)?)?;
            info!("done creating yaml configuration: {:?}", f_out);
            Ok(())
        } else {
            Err(RuntimeError::ParseError("no networkmap configuration found".to_owned()))
        }
    }

    #[cfg(feature = "sync")]
    pub async fn sync_config(&mut self) -> Result<(), RuntimeError> {

        debug_assert!(self.pull_config || self.push_config);
        
        // let mut proc = nm.to_ssh(target, subnet, command, extra_options)
        let nm = self.load_cfg()?.networkmap;
        let target = nm.get_sync_host().ok_or(RuntimeError::SyncError("no sync host has been set".to_owned()))?;
        let push = if self.push_config {
            // load config
            Some(self.load_cfg()?)
        } else {
            None
        };
        let proc = nm.to_ssh_sync(target, None, push.is_some()).await;
        match push {
            Some(c) => {
                let captured = proc 
                    .exec()
                    .stdin(serde_yml::to_string(&c).unwrap().as_str())
                    .capture();
                
                match captured {
                    Ok(e) => {
                        if e.success() {
                            info!("configuration pushed successfully");
                            Ok(())
                        } else {
                            Err(RuntimeError::SyncError("configuration push failed".to_owned()))
                        }
                    }
                    Err(e) => {
                        Err(RuntimeError::SpawnError(proc.to_string(), e.to_string()))
                    }
                }
            }
            None => {
                let output = proc
                    .exec()
                    .stdout(subprocess::Redirection::Pipe)
                    .capture()
                    .map_err(|e| RuntimeError::SpawnError(proc.to_string(), e.to_string()))?;
                let output = if output.success() {
                    output.stdout_str()
                } else {
                    return Err(RuntimeError::SyncError("configuration pull failed".to_owned()));
                };
                let c: Config = serde_yml::from_str(&output)?;
                #[cfg(debug_assertions)]
                debug!("configuration pulled: {:?}", c);
                #[cfg(not(debug_assertions))]
                {
                    let target = self.config
                        .as_ref()
                        .map(PathBuf::from)
                        .unwrap_or(Config::default_path(Some(home::home_dir().ok_or(RuntimeError::NoSuchFile("couldn't determine home directory".to_owned()))?)));
                    debug!("target file is {:?}", target);
                    if let Some(p) = target.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)?;
                        }
                    }
                    let mut output = std::fs::File::create(&target)?;
                    write!(output, "{}", serde_yml::to_string(&c)?)?;
                }
                #[cfg(debug_assertions)]
                {
                    warn!("not writing to disk: not in release mode");
                    println!("{}", serde_yml::to_string(&c)?);
                }
                info!("successfully pulled configuration");
                Ok(())
            }
        }
    }
}
