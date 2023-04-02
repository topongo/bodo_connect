#![feature(future_join)]

mod ssh;
mod waker;
mod net;

use std::future::join;
use std::net::IpAddr;
use std::str::FromStr;
use reqwest::Method;

use crate::net::*;
use crate::ssh::process::SSHProcess;
use crate::waker::Waker;
// use crate::router::Router;


#[tokio::main]
async fn main() {
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

    let a_waker = nm.wake(target);
    let a_proc = nm.to_ssh(target, subnet, Vec::new());

    let (waker, mut proc): (Result<(), String>, SSHProcess) = join!(a_waker, a_proc).await;

    if let Err(s) = waker {
        panic!("waker erorr: {}", s)
    }

    eprintln!("{}", proc.get_args().join(" "));
    proc.run(None).expect("ssh error");
}

