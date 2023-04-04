mod networkmap;
mod host;
mod subnet;
mod ip;
mod ssh;

use std::net::IpAddr;
use networkmap::NetworkMap;
use subnet::Subnet;
use host::Host;
use std::str::FromStr;
use external_ip::get_ip;
use reachable::{ResolvePolicy, Target, TcpTarget};
use futures::executor::block_on;
// use crate::router::Router;


#[tokio::main]
async fn main() {
    let mut nm = NetworkMap::new();
    
    // === redacted ===

    let subdomain = match nm.find_current_subnet().await {
        Some(s) => {
            println!("You are in this subnet: {:?}", s);
            Some(s)
        },
        None => {
            println!("can't find known subnet");
            None
        }
    };

    nm.command_gen(nm.get_host("dell").unwrap(), subdomain).await;
}

