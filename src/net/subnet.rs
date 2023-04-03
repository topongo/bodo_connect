use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use crate::net::Host;
use std::net::IpAddr;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Subnet {
    // identity
    pub subdomain: String,
    hosts: HashMap<String, Host>,
    pub eip: Option<IpAddr>
}

impl Subnet {
    pub fn new(subdomain: String, eip: Option<IpAddr>) -> Subnet {
        Subnet { subdomain, eip, hosts: HashMap::new() }
    }

    pub fn add_host(&mut self, h: Host) {
        self.hosts.insert(h.name.clone(), h);
    }

    // pub fn has_host(&self, q: &String) -> bool {
    //     self.hosts.contains_key(q)
    // }

    pub fn get_host(&self, q: &str) -> Option<&Host> {
        self.hosts.get(q)
    }

    pub fn get_hosts(&self) -> Vec<&Host> {
        self.hosts.values().collect()
    }

    pub fn get_master(&self) -> &Host {
        match self.hosts.values().find(|h| h.is_master()) {
            Some(h) => h,
            None => panic!("subnet {} has no master", self.subdomain)
        }
    }

    pub fn has_host(&self, h: &Host) -> bool {
        self.hosts.contains_key(&h.name)
    }
}

impl Debug for Subnet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subnet {{ subdomain: \"{}\", eip: {:?}, hosts: {} }}", self.subdomain, self.eip, self.hosts.len())
    }
}
