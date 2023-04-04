use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use external_ip::get_ip;
use reachable::{IcmpTarget, ResolvePolicy, Status, Target, TcpTarget};
use crate::ssh::hop::Hop;
use crate::Host;
use crate::subnet::Subnet;


const CLOUD_FLARE: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

fn get_resolve_policy(i: IpAddr) -> ResolvePolicy {
    match i {
        IpAddr::V4(..) => ResolvePolicy::ResolveToIPv4,
        IpAddr::V6(..) => ResolvePolicy::ResolveToIPv6
    }
}

#[derive(Debug)]
pub(crate) struct NetworkMap {
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
        self.subnets.values().find(|s| { s.eip.is_some() && s.eip.unwrap() == ip })
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

    pub async fn hops_gen(&self, target: &Host, subnet: Option<&Subnet>) -> Vec<Hop> {
        match subnet {
            Some(_) => {
                // known subnet
                vec![target.get_hop(None)]
            },
            None => {
                // unknown subnet
                let target_subnet = self.get_host_subnet(target);
                let master = target_subnet.get_master();
                vec![master.get_hop(Some(target_subnet))]
            }
        }
    }

    pub async fn command_gen(&self, target: &Host, subnet: Option<&Subnet>) {
        let hops = self.hops_gen(target, subnet).await;
        println!("{:?}", hops);
    }
}
