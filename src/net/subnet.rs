#![allow(dead_code)]

use std::fmt::{Debug, Formatter};
use std::net::IpAddr;
use serde::Deserialize;

use crate::net::Host;

#[derive(Deserialize)]
pub struct Subnet {
    // identity
    pub subdomain: String,
    hosts: Vec<Host>,
    pub eip: Option<IpAddr>
}

impl Subnet {
    pub fn new(subdomain: String, eip: Option<IpAddr>) -> Subnet {
        Subnet { subdomain, eip, hosts: Vec::new() }
    }

    pub fn add_host(&mut self, h: Host) {
        self.hosts.push(h);
    }

    pub fn get_host(&self, q: &str) -> Option<&Host> {
        self.hosts.iter().find(|h| h.name == q)
    }

    pub fn get_hosts(&self) -> Vec<&Host> {
        self.hosts.iter().collect()
    }

    pub fn get_master(&self) -> &Host {
        match self.hosts.iter().find(|h| h.is_master()) {
            Some(h) => h,
            None => panic!("subnet {} has no master", self.subdomain)
        }
    }

    pub fn has_host(&self, h: &Host) -> bool {
        self.hosts.contains(&h)
    }
}

impl Debug for Subnet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subnet {{ subdomain: \"{}\", eip: {:?}, hosts: {} }}", self.subdomain, self.eip, self.hosts.len())
    }
}

impl PartialEq for Subnet {
    fn eq(&self, other: &Self) -> bool {
        self.subdomain.eq(&other.subdomain)
    }
}
