use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::Duration;
use external_ip::get_ip;
use reachable::{IcmpTarget, ResolvePolicy, Status, Target, TcpTarget};
use subprocess::ExitStatus;
use crate::ssh::{*, options::*};
use crate::net::{Host, Subnet};
use crate::waker::Waker;


const CLOUD_FLARE: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

fn get_resolve_policy(i: IpAddr) -> ResolvePolicy {
    match i {
        IpAddr::V4(..) => ResolvePolicy::ResolveToIPv4,
        IpAddr::V6(..) => ResolvePolicy::ResolveToIPv6
    }
}

#[derive(Debug)]
pub struct NetworkMap {
    subnets: HashMap<String, Subnet>
}

impl NetworkMap {
    pub fn new() -> NetworkMap {
        NetworkMap { subnets: HashMap::new() }
    }

    pub fn add_subnet(&mut self, s: Subnet) {
        self.subnets.insert(s.subdomain.clone(), s);
    }

    pub fn get_host(&self, q: &str) -> Option<&Host> {
        for s in self.subnets.values() {
            match s.get_host(q) {
                None => {},
                Some(h) => { return Some(h) }
            }
        }
        None
    }

    pub fn is_available(ip: IpAddr, port: Option<u16>) -> bool {
        // return false;
        let rp = get_resolve_policy(ip);
        match match port {
            Some(p) => TcpTarget::new(
                ip.to_string(),
                p,
                Duration::from_millis(500),
                rp
            ).check_availability(),
            None => IcmpTarget::new(
                ip.to_string(),
                rp
            ).check_availability()
        } {
            Ok(r) => {
                match r {
                    Status::Available => true,
                    _ => false
                }
            },
            Err(_) => false
        }
    }

    pub fn get_subnet_by_ip(&self, ip: IpAddr) -> Option<&Subnet> {
        self.subnets.values().find(|s| {
            if s.eip.is_some() {
                s.eip.unwrap() == ip
            } else {
                format!("{}:0", s.subdomain)
                    .to_socket_addrs().unwrap()
                    .next().unwrap()
                    .ip() == ip
            }
        })
    }

    pub fn get_masters(&self) -> Vec<(&Subnet, &Host)> {
        self.subnets
            .values()
            .map(|s| (s, s.get_master()))
            .collect()
    }

    /// Gets master of given host
    pub fn get_host_subnet(&self, h: &Host) -> &Subnet {
        for s in self.subnets.values() {
            if s.has_host(h) {
                return s
            }
        }
        panic!("host is not in any subnet")
    }

    pub fn get_host_master(&self, h: &Host) -> &Host {
        self.get_host_subnet(h).get_master()
    }

    /// Gets client external ip and returns the optional matched subnet in which the client is.
    pub async fn find_current_subnet(&self) -> Option<&Subnet> {
        // are we online?
        if NetworkMap::is_available(CLOUD_FLARE, Some(80)) {
            // yes, get client external ip
            match get_ip().await {
                Some(client_eip) => self.get_subnet_by_ip(client_eip),
                None => None
            }
        } else {
            // no, check if some network master is available
            for (s, m) in self.get_masters() {
                if NetworkMap::is_available(m.ip, Some(m.port)) {
                    return Some(s)
                } else {
                    continue
                }
            }
            None
        }
    }

    pub async fn hops_gen(&self, target: &Host, subnet: Option<&Subnet>) -> Option<Vec<Hop>> {
        match subnet {
            Some(_) => {
                // known subnet
                None
            },
            None => {
                // unknown subnet
                let target_subnet = self.get_host_subnet(target);
                let master = target_subnet.get_master();
                Some(vec![master.get_hop(Some(target_subnet))])
            }
        }
    }

    pub async fn to_ssh(&self, target: &Host, subnet: Option<&Subnet>, mut command: Vec<String>) -> SSHProcess {
        let target_string = target.identity_string();

        let mut options = SSHOptionStore::default();

        if let Some(hops) = self.hops_gen(target, subnet).await {
            options.add_option(Box::new(JumpHosts::new(hops)));
        }
        if let Some(p_o) = target.port_option() {
            options.add_option(Box::new(p_o))
        }

        let mut output = vec!["ssh".to_string()];
        output.append(&mut options.args_gen());
        output.push(target_string);
        output.append(&mut command);
        SSHProcess::new(output)
    }

    pub async fn wake(&self, target: &Host) -> Result<(), String> {
        match &target.waker {
            None => {
                println!("asked for wake but this target hasn't a waker");
                Ok(())
            },
            Some(w) => match w {
                Waker::HttpWaker(method, url) => {
                    let client = reqwest::Client::new();
                    println!("making http request to {}", url);
                    match client.request(method.clone(), url).send().await {
                        Ok(res) => {
                            if res.status() == 200 {
                                Ok(())
                            } else {
                                Err(format!("http error: {}", res.status()))
                            }
                        },
                        Err(e) => {
                            Err(format!("http error: {}", e))
                        }
                    }
                },
                Waker::WolWaker(mac) => {
                    let master = self.get_host_master(target);
                    match self.to_ssh(master, None, vec!["wol".to_string(), mac.to_string()]).await.run_stdout_to_stderr() {
                        Ok(e) => {
                            if let ExitStatus::Exited(n) = e {
                                if n == 0 {
                                    Ok(())
                                } else {
                                    Err(format!("ssh waker exited with code {}", n))
                                }
                            } else {
                                Err(format!("ssh waker ended: {:?}", e))
                            }
                        },
                        Err(e) => Err(format!("{:?}", e))
                    }
                }
            }
        }
    }
}
