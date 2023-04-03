use std::fs::File;
use log::{debug, error, info, LevelFilter};
use clap::Parser;
use std::future::join;
use std::io::{BufReader, Read};
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use reqwest::Method;


use crate::net::*;
use crate::waker::Waker;
use crate::logger;
use crate::net::networkmap_prelude::NetworkMapPrelude;

#[derive(Parser, Debug)]
#[command(
    name = "bodoConnect",
    about = "create ssh command on the fly no matter in which network you are"
)]
pub struct BodoConnect {
    #[arg(long, help = "Select different networkmap.json file")]
    networkmap: Option<PathBuf>,
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

impl BodoConnect {
    pub async fn main(&self) -> i32 {
        log::set_logger(&logger::CONSOLE_LOGGER).unwrap();
        log::set_max_level(
            match self.debug {
                v if v >= 2 => LevelFilter::Debug,
                1 => LevelFilter::Info,
                _ => LevelFilter::Warn
            }
        );

        let np: NetworkMapPrelude = match File::open(File::open(match match &self.networkmap {
            Some(p) => if p.exists() && (p.is_file() || p.is_symlink()) {
                p
            } else {
                error!("no such file or directory: {:?}", p);
                return 1
            }
            None => {
                &PathBuf::from("~/.config/bodoConnect/networkmap.json")
            }
        }.to_str() {
            Some(p) => p,
            None => {
                error!("invalid path: {:?}", p);
                return 1
            }
        }) {
            Ok(mut f) => f,
            Err(e) => {
                error!("cannot open networkmap file: {:?}", e);
                return 2
            }
        })).read;

        self.static_testing().await
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

        // let a_waker = nm.wake(target);
        // let a_proc = nm.to_ssh(target, subnet, Vec::new());
        //
        // let (waker, mut proc): (Result<(), String>, SSHProcess) = join!(a_waker, a_proc).await;
        //
        // if let Err(s) = waker {
        //     panic!("waker erorr: {}", s)
        // }

        // eprintln!("{}", proc.get_args().join(" "));
        // proc.run(None).expect("ssh error");
    }
}
