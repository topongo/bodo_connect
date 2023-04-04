use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::Duration;
use external_ip::get_ip;
#[cfg(feature = "log")]
use log::{debug, info};
#[cfg(not(feature = "log"))]
use crate::{debug, info};
use reachable::{IcmpTarget, ResolvePolicy, Status, Target, TcpTarget};

use crate::ssh::{*, options::*};
use crate::net::{Host, Subnet};
#[cfg(feature = "wake")]
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
        let target_subnet = self.get_host_subnet(target);
        let master = target_subnet.get_master();
        let hops = vec![master.get_hop(Some(target_subnet))];

        match subnet {
            Some(s) => {
                // known subnet
                if target_subnet == s {
                    None
                } else {
                    Some(hops)
                }
            },
            None => {
                // unknown subnet
                Some(hops)
            }
        }
    }

    pub async fn to_ssh(&self, target: &Host, subnet: Option<&Subnet>, command: &mut Vec<String>, eoptions: Option<SSHOptionStore>) -> SSHProcess {
        debug!("generating target string");
        let target_string = target.identity_string();
        info!("host string genetared: {}", target_string);

        debug!("generating ssh options");
        let mut options = SSHOptionStore::default();

        if let Some(hops) = self.hops_gen(target, subnet).await {
            debug!("hops required, adding to options: {:?}", hops.iter().map(|h| h.to_string()).collect::<Vec<String>>());
            options.add_option(Box::new(JumpHosts::new(hops)));
        }
        if let Some(p_o) = target.port_option() {
            debug!("port specification needed, adding to options: {:?}", p_o);
            options.add_option(Box::new(p_o))
        }
        info!("ssh options generated: {:?}", options.args_gen());

        if let Some(o) = eoptions {
            debug!("extra options present, merging");
            options.merge(o);
        }

        debug!("generating ssh command");
        let mut output = vec!["ssh".to_string()];
        output.append(&mut options.args_gen());
        output.push(target_string);
        output.append(command);
        SSHProcess::new(output)
    }

    #[cfg(feature = "wake")]
    pub async fn wake(&self, target: &Host) -> Result<(), String> {
        match &target.waker {
            None => {
                info!("won't wake host since it hasn't any waker");
                Ok(())
            },
            Some(w) => match w {
                Waker::HttpWaker { method, url} => {
                    info!("making {} request to {}", method, url);
                    let client = reqwest::Client::new();
                    match client.request(method.clone(), url).send().await {
                        Ok(res) => {
                            debug!("status code of request: {}", res.status());
                            if res.status() == 200 {
                                Ok(())
                            } else {
                                Err(format!("http error: {}", res.status()))
                            }
                        },
                        Err(e) => {
                            Err(format!("request error: {}", e))
                        }
                    }
                },
                Waker::WolWaker { mac} => {
                    info!("waking host with mac {} through ssh", mac);
                    let master = self.get_host_master(target);
                    info!("master to execute wake on is {}", master.name);
                    debug!("generating ssh command for wake operation");
                    let mut wake_proc = self.to_ssh(master, None, &mut vec!["wol".to_string(), mac.to_string()], None).await;
                    debug!("ssh waker command is `{}`", wake_proc.to_string());
                    match wake_proc.run_stdout_to_stderr() {
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

impl TryFrom<Vec<Subnet>> for NetworkMap {
    type Error = String;

    fn try_from(value: Vec<Subnet>) -> Result<Self, Self::Error> {
        let mut n = NetworkMap::new();
        let mut subs = HashSet::new();
        for s in value.into_iter() {
            if subs.contains(&s.subdomain) {
                return Err(s.subdomain)
            } else {
                subs.insert(s.subdomain.clone());
                n.add_subnet(s);
            }
        }
        Ok(n)
    }
}
